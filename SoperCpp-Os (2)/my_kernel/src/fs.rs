const MAX_FILES: usize = 20;
const MAX_PATH_LEN: usize = 48;

#[derive(Clone, Copy)]
pub struct VirtualFile {
    pub path: [u8; MAX_PATH_LEN],
    pub path_len: usize,
    pub exists: bool,
}

pub struct FileSystem {
    files: [VirtualFile; MAX_FILES],
}

impl FileSystem {
    pub const fn new() -> Self {
        FileSystem {
            files: [VirtualFile {
                path: [0; MAX_PATH_LEN],
                path_len: 0,
                exists: false,
            }; MAX_FILES],
        }
    }

    // تهيئة المجلدات الأساسية للنظام عند الإقلاع
    pub fn init_defaults(&mut self) {
        self.create_path(b"/app");
        self.create_path(b"/home");
        self.create_path(b"/os");
    }

    // إضافة مسار جديد (مجلد أو ملف)
    pub fn create_path(&mut self, path_bytes: &[u8]) -> bool {
        // التحقق من عدم تكرار المسار مسبقاً
        for file in self.files.iter() {
            if file.exists && &file.path[..file.path_len] == path_bytes {
                return false; // موجود بالفعل
            }
        }

        for file in self.files.iter_mut() {
            if !file.exists {
                let len = if path_bytes.len() > MAX_PATH_LEN { MAX_PATH_LEN } else { path_bytes.len() };
                file.path = [0; MAX_PATH_LEN];
                file.path[..len].copy_from_slice(&path_bytes[..len]);
                file.path_len = len;
                file.exists = true;
                return true;
            }
        }
        false // الذاكرة ممتلئة
    }

    // الحصول على قائمة الملفات لعرضها
    pub fn get_files(&self) -> &[VirtualFile; MAX_FILES] {
        &self.files
    }
}