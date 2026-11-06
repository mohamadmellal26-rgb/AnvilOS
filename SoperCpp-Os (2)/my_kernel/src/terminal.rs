use crate::WRITER;
use crate::fs::FileSystem;
use pc_keyboard::DecodedKey;

const MAX_COMMAND_LENGTH: usize = 70;
const MAX_DIR_LEN: usize = 32;

pub struct Terminal {
    command_buffer: [u8; MAX_COMMAND_LENGTH],
    buffer_index: usize,
    fs: FileSystem,
    current_dir: [u8; MAX_DIR_LEN],
    current_dir_len: usize,
}

impl Terminal {
    pub const fn new() -> Self {
        let mut dir = [0; MAX_DIR_LEN];
        let default_path = b"/home";
        let mut i = 0;
        while i < default_path.len() {
            dir[i] = default_path[i];
            i += 1;
        }

        Terminal {
            command_buffer: [0; MAX_COMMAND_LENGTH],
            buffer_index: 0,
            fs: FileSystem::new(),
            current_dir: dir,
            current_dir_len: default_path.len(),
        }
    }

    pub fn init(&mut self) {
        self.fs.init_defaults();

        {
            let mut writer = WRITER.lock();
            // ضبط موقع البداية للسطر الأول في التيرمنال عبر الأعمدة والأسطر
            writer.row_position = 2;
            writer.column_position = 2;
            writer.write_string("SoperCpp-Os Terminal v1.3 (Auto-Home)\n");
        } 
        
        self.print_prompt();
    }

    pub fn print_prompt(&self) {
        let mut writer = WRITER.lock();
        writer.write_string("SoperCpp:");
        for &b in &self.current_dir[..self.current_dir_len] {
            let s = [b];
            if let Ok(utf8_str) = core::str::from_utf8(&s) {
                writer.write_string(utf8_str);
            }
        }
        writer.write_string("/# ");
    }

    pub fn handle_key(&mut self, decoded_key: DecodedKey) {
        match decoded_key {
            DecodedKey::Unicode(character) => {
                match character {
                    '\n' => {
                        {
                            let mut writer = WRITER.lock();
                            writer.write_string("\n");
                        }
                        
                        self.execute_command();
                        self.buffer_index = 0;
                        self.print_prompt();
                    }
                    '\x08' | '\u{7F}' => {
                        if self.buffer_index > 0 {
                            self.buffer_index -= 1;
                            let mut writer = WRITER.lock();
                            writer.write_string("\x08"); 
                        }
                    }
                    c if c >= ' ' && c <= '~' => {
                        if self.buffer_index < MAX_COMMAND_LENGTH - 1 {
                            self.command_buffer[self.buffer_index] = c as u8;
                            self.buffer_index += 1;
                            
                            let mut writer = WRITER.lock();
                            let mut s = [0u8; 4];
                            let utf8_str = c.encode_utf8(&mut s);
                            writer.write_string(utf8_str);
                        }
                    }
                    _ => {}
                }
            }
            DecodedKey::RawKey(_) => {}
        }
    }

    fn execute_command(&mut self) {
        if self.buffer_index == 0 {
            return;
        }

        let cmd_slice = &self.command_buffer[..self.buffer_index];

        let mut space_idx = None;
        for (i, &b) in cmd_slice.iter().enumerate() {
            if b == b' ' {
                space_idx = Some(i);
                break;
            }
        }

        let (command, args) = match space_idx {
            Some(idx) => (&cmd_slice[..idx], &cmd_slice[idx + 1..]),
            None => (cmd_slice, &[][..]),
        };

        let mut writer = WRITER.lock();

        if command == b"help" {
            writer.write_string("Available commands:\n");
            writer.write_string("  help           - Display help info\n");
            writer.write_string("  clear          - Clear screen\n");
            writer.write_string("  uname          - Print OS name\n");
            writer.write_string("  ls             - List all files and directories\n");
            writer.write_string("  touch <name>   - Create file in current dir\n");
        } else if command == b"clear" {
            writer.clear_screen(0x00000080);
        } else if command == b"uname" {
            writer.write_string("SoperCpp-Os x86_64 (Custom Kernel)\n");
        } else if command == b"ls" {
            writer.write_string("FileSystem Paths:\n");
            let mut found = false;
            for file in self.fs.get_files().iter() {
                if file.exists {
                    found = true;
                    writer.write_string("  ");
                    for &b in &file.path[..file.path_len] {
                        let s = [b];
                        if let Ok(utf8_str) = core::str::from_utf8(&s) {
                            writer.write_string(utf8_str);
                        }
                    }
                    writer.write_string("\n");
                }
            }
            if !found {
                writer.write_string("  (empty)\n");
            }
        } else if command == b"touch" {
            if args.is_empty() {
                writer.write_string("Error: missing filename. Usage: touch <filename>\n");
            } else {
                let mut full_path = [0; MAX_DIR_LEN + 32];
                let mut p_len = 0;

                let starts_with_slash = !args.is_empty() && args[0] == b'/';

                if !starts_with_slash {
                    for i in 0..self.current_dir_len {
                        full_path[p_len] = self.current_dir[i];
                        p_len += 1;
                    }

                    if p_len == 0 || full_path[p_len - 1] != b'/' {
                        full_path[p_len] = b'/';
                        p_len += 1;
                    }
                }

                for i in 0..args.len() {
                    if p_len < full_path.len() {
                        full_path[p_len] = args[i];
                        p_len += 1;
                    }
                }

                let success = self.fs.create_path(&full_path[..p_len]);
                if success {
                    writer.write_string("File created in current directory.\n");
                } else {
                    writer.write_string("Error: path already exists or storage full.\n");
                }
            }
        } else {
            writer.write_string("Unknown command. Type 'help'\n");
        }
        
        for i in 0..MAX_COMMAND_LENGTH {
            self.command_buffer[i] = 0;
        }
    }
}