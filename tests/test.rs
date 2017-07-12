extern crate byteorder;
extern crate ghakuf;

use byteorder::{BigEndian, WriteBytesExt};
use ghakuf::formats::*;
use ghakuf::messages::*;
use ghakuf::reader::*;
use ghakuf::writer::writer::*;
use std::fs::{OpenOptions, File};
use std::io::prelude::*;
use std::io::Read;
use std::path::PathBuf;

#[test]
fn parse_integration_testing() {
    make_smf_sample(PathBuf::from("tests/read.mid"));
    let mut reader = Reader::new(
        Box::new(ReaderHandler { messages: test_messages() }),
        PathBuf::from("tests/read.mid"),
    ).unwrap();
    assert!(reader.read().is_ok());
}
struct ReaderHandler {
    messages: Vec<Message>,
}
impl Handler for ReaderHandler {
    fn header(&mut self, format: Format, track: u16, time_base: u16) {
        assert_eq!(format, Format::F1);
        assert_eq!(track, 2);
        assert_eq!(time_base, 480);
    }
    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
        assert_eq!(
            Message::MetaEvent {
                delta_time: VLQ::new(delta_time),
                event: event.clone(),
                data: data.clone(),
            },
            self.messages[0]
        );
        self.messages.remove(0);
    }
    fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
        assert_eq!(
            Message::MidiEvent {
                delta_time: VLQ::new(delta_time),
                event: event.clone(),
            },
            self.messages[0]
        );
        self.messages.remove(0);
    }
    fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
        assert_eq!(
            Message::SysExEvent {
                delta_time: VLQ::new(delta_time),
                event: event.clone(),
                data: data.clone(),
            },
            self.messages[0]
        );
        self.messages.remove(0);
    }
    fn track_change(&mut self) {
        if self.messages.len() > 0 && self.messages[0] == Message::TrackChange {
            self.messages.remove(0);
        }
    }
}

#[test]
fn build_integration_testing() {
    let mut writer = Writer::new();
    writer.running_status(true);
    let test_messages = test_messages();
    for message in test_messages {
        writer.push(message);
    }
    assert!(writer.write(PathBuf::from("tests/write.mid")).is_ok());
    let mut data_write = Vec::new();
    let mut f = File::open("tests/write.mid").expect("Unable to open file");
    f.read_to_end(&mut data_write).unwrap();
    make_smf_sample(PathBuf::from("tests/write_teacher.mid"));
    let mut data_read = Vec::new();
    let mut f = File::open("tests/write_teacher.mid").expect("Unable to open file");
    f.read_to_end(&mut data_read).unwrap();
    if data_read.len() == 0 || data_write.len() == 0 {
        assert!(false);
    }
    assert_eq!(data_read, data_write);
}

fn make_smf_sample(path: PathBuf) {
    let _ = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .and_then(|mut f| {
            f.write_all(b"MThd")?; //HEADER DATA
            f.write_all(&[0, 0, 0, 6, 0, 1, 0, 2])?;
            f.write_u16::<BigEndian>(480)?;
            f.write_all(b"MTrk")?; //TRACK DATA
            f.write_u32::<BigEndian>(11)?;
            f.write_all(&[0, 0xFF, 0x51, 0x03])?; //SET TEMPO
            let tempo: u32 = 60 * 1000000 / 102; //bpm:102
            f.write_all(
                &[(tempo >> 16) as u8, (tempo >> 8) as u8, tempo as u8],
            )?;
            f.write_all(&[0, 0xFF, 0x2F, 0])?;
            f.write_all(b"MTrk")?; //TRACK DATA
            f.write_u32::<BigEndian>(24)?;
            f.write_all(&[0, 0x90, 0x3C, 0x7F])?; //NOTE ON
            f.write_all(&[0x30, 0x3C, 0])?; //NOTE OFF
            f.write_all(&[0, 0x3E, 0x7F])?; //NOTE ON
            f.write_all(&[0x30, 0x3E, 0])?; //NOTE OFF
            f.write_all(&[0, 0x40, 0x7F])?; //NOTE ON
            f.write_all(&[0x81, 0x40, 0x40, 0])?; //NOTE OFF
            f.write_all(&[0, 0xFF, 0x2F, 0x00])?; //TRACK END
            Ok(())
        });
}

fn test_messages() -> Vec<Message> {
    let mut test_messages: Vec<Message> = Vec::new();
    let tempo: u32 = 60 * 1000000 / 102; //bpm:102
    test_messages.push(Message::MetaEvent {
        delta_time: VLQ::new(0),
        event: MetaEvent::SetTempo,
        data: [(tempo >> 16) as u8, (tempo >> 8) as u8, tempo as u8].to_vec(),
    });
    test_messages.push(Message::MetaEvent {
        delta_time: VLQ::new(0),
        event: MetaEvent::EndOfTrack,
        data: Vec::new(),
    });
    test_messages.push(Message::TrackChange);
    test_messages.push(Message::MidiEvent {
        delta_time: VLQ::new(0),
        event: MidiEvent::NoteOn {
            ch: 0,
            note: 0x3c,
            velocity: 0x7f,
        },
    });
    test_messages.push(Message::MidiEvent {
        delta_time: VLQ::new(48),
        event: MidiEvent::NoteOn {
            ch: 0,
            note: 0x3c,
            velocity: 0,
        },
    });
    test_messages.push(Message::MidiEvent {
        delta_time: VLQ::new(0),
        event: MidiEvent::NoteOn {
            ch: 0,
            note: 0x3e,
            velocity: 0x7f,
        },
    });
    test_messages.push(Message::MidiEvent {
        delta_time: VLQ::new(48),
        event: MidiEvent::NoteOn {
            ch: 0,
            note: 0x3e,
            velocity: 0,
        },
    });
    test_messages.push(Message::MidiEvent {
        delta_time: VLQ::new(0),
        event: MidiEvent::NoteOn {
            ch: 0,
            note: 0x40,
            velocity: 0x7f,
        },
    });
    test_messages.push(Message::MidiEvent {
        delta_time: VLQ::new(192),
        event: MidiEvent::NoteOn {
            ch: 0,
            note: 0x40,
            velocity: 0,
        },
    });
    test_messages.push(Message::MetaEvent {
        delta_time: VLQ::new(0),
        event: MetaEvent::EndOfTrack,
        data: Vec::new(),
    });
    test_messages
}
