use crate::FramebufferWriter;

pub struct DesktopGUI;

impl DesktopGUI {
    /// رسم سطح المكتب بخلفية سوداء، وشريط مهام أبيض أنيق بدون أي لون أزرق، وبدون أي عناصر في اليمين
    pub fn draw_desktop(writer: &mut FramebufferWriter) {
        let width = writer.width;
        let height = writer.height;

        let taskbar_height = 55;
        let desktop_height = height - taskbar_height;

        // 1. رسم خلفية سطح المكتب (لون أسود ناصع)
        let bg_color = 0x00000000; // Pure Black

        for y in 0..desktop_height {
            for x in 0..width {
                writer.draw_pixel(x, y, bg_color);
            }
        }

        // 2. رسم شريط المهام السفلي (لون أبيض ناصع)
        let taskbar_color = 0x00FFFFFF; // Pure White
        let taskbar_start_y = desktop_height;

        for y in taskbar_start_y..height {
            for x in 0..width {
                writer.draw_pixel(x, y, taskbar_color);
            }
        }

        // رسم خط فاصل رمادي خفيف أعلى شريط المهام لتحديد الحافة بوضوح على الخلفية السوداء
        let border_color = 0x00CBD5E1; // Light Gray Border
        for x in 0..width {
            writer.draw_pixel(x, taskbar_start_y, border_color);
        }

        // 3. زر البدء في أقصى اليسار (بتصميم أنيق بلون رمادي داكن فخم، وخالٍ تماماً من اللون الأزرق)
        let btn_x = 15;
        let btn_y = taskbar_start_y + 10;
        let btn_w = 90;
        let btn_h = 35;
        let btn_color = 0x001E293B; // رمادي داكن / فحمى أنيق
        let btn_border = 0x000F172A; // إطار داكن متناسق

        for y in btn_y..(btn_y + btn_h) {
            for x in btn_x..(btn_x + btn_w) {
                if x == btn_x || x == btn_x + btn_w - 1 || y == btn_y || y == btn_y + btn_h - 1 {
                    writer.draw_pixel(x, y, btn_border);
                } else {
                    writer.draw_pixel(x, y, btn_color);
                }
            }
        }

        // 4. الجهة اليمنى: تم إزالة صندوق الساعة وكل ما يتعلق باليمين تماماً بناءً على طلبك.
    }
}