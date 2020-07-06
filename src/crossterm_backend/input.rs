use log::info;
use std::sync::mpsc::{Sender, SendError};
use std::collections::HashMap;
use crossterm::event::{read, Event, KeyModifiers, KeyCode, KeyEvent};

pub fn wait_for_keypress(tx: Sender<HashMap<String, String>>) -> Result<(), SendError<HashMap<String, String>>> {
    loop {
        let console = "console".to_string();
        match read().unwrap() {
            Event::Key(event) => {
                match event {
                    // CTRL_C
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL
                    } => {
                        // exit...
                        info!("Shutting down...");
                        let mut h = HashMap::new();
                        h.insert("system".to_string(), "\\u001bQ".to_string());
                        tx.send(h)?;
                        break; // exit the loop and stop accepting input
                    },
                    // CTRL_U
                    KeyEvent {
                        code: KeyCode::Char('u'),
                        modifiers: KeyModifiers::CONTROL
                    } => {
                        info!("Clear buffer");
                        let mut h = HashMap::new();
                        h.insert(console, "\\u001bU".to_string());
                        tx.send(h)?;
                    },
                    // ENTER
                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE
                    } => {
                        let mut h = HashMap::new();
                        h.insert(console, "\n".to_string());
                        tx.send(h)?;
                    },
                    // General key press
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE
                    } => {
                        let mut h = HashMap::new();
                        h.insert(console, c.to_string());
                        tx.send(h)?;
                    },
                    // I don't care about anything else
                    _ => {}
                }
                info!("Key Event: {:?}", event);
            },
            _ => {} // I don't care about these events.
        }
    }
    Ok(())
}