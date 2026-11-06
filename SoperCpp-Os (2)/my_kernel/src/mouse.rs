use x86_64::instructions::port::Port;
use crate::WRITER;

pub struct Mouse {
    packet_index: usize,
    packet: [u8; 4], // دعم حتى 4 بايتات في حال تفعيل وضع الـ Intellimouse
    pub x: i32,
    pub y: i32,
    pub prev_x: i32,
    pub prev_y: i32,
    pub left_click: bool,
    pub right_click: bool,
}

impl Mouse {
    pub const fn new() -> Self {
        Mouse {
            packet_index: 0,
            packet: [0; 4],
            x: 640,  // منتصف الشاشة أفقيّاً (1280 / 2)
            y: 360,  // منتصف الشاشة عمودياً (720 / 2)
            prev_x: 640,
            prev_y: 360,
            left_click: false,
            right_click: false,
        }
    }

    unsafe fn wait_write(&self, command_port: &mut Port<u8>) {
        for _ in 0..10000 {
            if (command_port.read() & 2) == 0 {
                break;
            }
        }
    }

    unsafe fn wait_read(&self, command_port: &mut Port<u8>) {
        for _ in 0..10000 {
            if (command_port.read() & 1) != 0 {
                break;
            }
        }
    }

    /// تهيئة تراك باد اللابتوب عبر PS/2 وتفعيل وضع الـ Intellimouse
    pub fn init(&mut self) {
        let mut command_port: Port<u8> = Port::new(0x64);
        let mut data_port: Port<u8> = Port::new(0x60);

        unsafe {
            // 1. تفعيل المنفذ المساعد (Auxiliary Mouse Device)
            self.wait_write(&mut command_port);
            command_port.write(0xA8u8);

            // 2. قراءة بايت الإعدادات
            self.wait_write(&mut command_port);
            command_port.write(0x20u8);
            self.wait_read(&mut command_port);
            let status: u8 = data_port.read();

            // 3. كتابة بايت الإعدادات مع تمكين مقاطعات الفأرة (IRQ 12)
            self.wait_write(&mut command_port);
            command_port.write(0x60u8);
            self.wait_write(&mut command_port);
            data_port.write(status | 0x02);

            // 4. إعدادات افتراضية
            self.write_mouse(&mut command_port, &mut data_port, 0xF6u8); 

            // 5. محاولة تفعيل تمديد عجلة التمرير / تراك باد اللابتوب (Intellimouse Mode sequence: F3 -> C8, F3 -> 64, F3 -> 50)
            self.write_mouse(&mut command_port, &mut data_port, 0xF3u8);
            self.write_mouse(&mut command_port, &mut data_port, 0xC8u8);
            self.write_mouse(&mut command_port, &mut data_port, 0xF3u8);
            self.write_mouse(&mut command_port, &mut data_port, 0x64u8);
            self.write_mouse(&mut command_port, &mut data_port, 0xF3u8);
            self.write_mouse(&mut command_port, &mut data_port, 0x50u8);

            // 6. تمكين البث (Enable Data Reporting)
            self.write_mouse(&mut command_port, &mut data_port, 0xF4u8);
        }

        self.draw_cursor();
    }

    unsafe fn write_mouse(&self, command_port: &mut Port<u8>, data_port: &mut Port<u8>, val: u8) {
        self.wait_write(command_port);
        command_port.write(0xD4u8); 
        
        self.wait_write(command_port);
        data_port.write(val);
        
        self.wait_read(command_port);
        let _: u8 = data_port.read();
    }

    /// معالجة الحزم القادمة من تراك باد اللابتوب
    pub fn handle_packet(&mut self, byte: u8) {
        // إزالة شرط المزامنة الصارم مؤقتاً لتجنب تجاهل حزم التراك باد
        self.packet[self.packet_index] = byte;
        self.packet_index += 1;

        // تراك باد اللابتوب قد يرسل 3 أو 4 بايتات
        if self.packet_index >= 3 {
            self.packet_index = 0;
            
            let flags = self.packet[0];
            let raw_dx = self.packet[1] as i32;
            let raw_dy = self.packet[2] as i32;

            if (flags & 0x40) != 0 || (flags & 0x80) != 0 {
                return; 
            }

            let dx = if (flags & 0x10) != 0 { raw_dx - 256 } else { raw_dx };
            let dy = if (flags & 0x20) != 0 { raw_dy - 256 } else { raw_dy };

            if dx != 0 || dy != 0 {
                self.prev_x = self.x;
                self.prev_y = self.y;
                // حركة سريعة ومتناسبة مع حساسية التراك باد
                self.x = (self.x + dx).clamp(0, 1279);
                self.y = (self.y - dy).clamp(0, 719); 
                self.redraw_cursor();
            }

            self.left_click = (flags & 0x01) != 0;
            self.right_click = (flags & 0x02) != 0;
        }
    }

    fn redraw_cursor(&self) {
        let mut writer = WRITER.lock();

        for row in 0..12 {
            for col in 0..8 {
                let px = self.prev_x as usize + col;
                let py = self.prev_y as usize + row;
                if px < writer.width && py < writer.height {
                    let bg_color = if py >= 665 { 0x00FFFFFF } else { 0x00000000 };
                    writer.draw_pixel(px, py, bg_color);
                }
            }
        }
        drop(writer);

        self.draw_cursor();
    }

    pub fn draw_cursor(&self) {
        let mut writer = WRITER.lock();
        
        const CURSOR_BITMAP: [u8; 12] = [
            0b10000000,
            0b11000000,
            0b11100000,
            0b11110000,
            0b11111000,
            0b11111100,
            0b11110000,
            0b10111000,
            0b00011100,
            0b00001100,
            0b00000110,
            0b00000110,
        ];

        for (row, &row_bits) in CURSOR_BITMAP.iter().enumerate() {
            for col in 0..8 {
                if (row_bits & (1 << (7 - col))) != 0 {
                    let px = self.x as usize + col;
                    let py = self.y as usize + row;
                    if px < writer.width && py < writer.height {
                        writer.draw_pixel(px, py, 0x00FFFFFF);
                    }
                }
            }
        }
    }
}