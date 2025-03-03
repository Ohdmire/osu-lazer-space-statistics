#![feature(windows_by_handle)]

use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::os::windows::fs::MetadataExt;
use std::path::Path;

fn get_lazer_location() -> Option<String> {

    // 获取当前用户的 AppData\Roaming 路径
    let appdata_roaming = match env::var("APPDATA") {
        Ok(path) => path,
        Err(_) => {
            eprintln!("无法获取 AppData\\Roaming 路径。");
            return None;
        }
    };

    // 构建 storage.ini 文件的路径
    let storage_ini_path = Path::new(&appdata_roaming).join("osu").join("storage.ini");

    // 读取 storage.ini 文件内容
    let target_folder = match read_storage_ini(&storage_ini_path) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("读取 storage.ini 文件失败: {}", err);
            return None;
        }
    };

    Some(target_folder)

}

fn main() {

    let target_folder = get_lazer_location().unwrap_or_else(|| { String::from("") });

    println!("从 storage.ini 读取的路径: {}", target_folder);

    println!("正在统计文件大小...");

    // 统计文件夹大小（包含硬链接和不包含硬链接）
    let (total_size_with_hardlinks, total_size_without_hardlinks) = calculate_folder_size(&target_folder);

    // 打印最大的单位
    println!("统计总大小（包含硬链接）: {}", format_size(total_size_with_hardlinks));
    println!("实际总大小（排除硬链接）: {}", format_size(total_size_without_hardlinks));

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("");

}

/// 将字节大小格式化为最大的单位（如 1.2G、3.8M）
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if size >= GB {
        format!("{:.1}G", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1}M", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1}K", size as f64 / KB as f64)
    } else {
        format!("{}B", size)
    }
}

/// 读取 storage.ini 文件并解析目标路径
fn read_storage_ini(path: &Path) -> io::Result<String> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);

    // 读取文件的第一行
    if let Some(first_line) = reader.lines().next() {
        let line = first_line?;
        // 路径在第一行
        let line = line.split('=').nth(1).unwrap().trim().to_string(); // 提取并去除空格
        Ok(line)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidData, "storage.ini 文件为空"))
    }
}

/// 统计文件夹大小（包含硬链接和不包含硬链接）
fn calculate_folder_size(folder: &str) -> (u64, u64) {
    let mut total_size_with_hardlinks = 0;
    let mut total_size_without_hardlinks = 0;

    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    // 递归处理子文件夹
                    let (sub_with_hardlinks, sub_without_hardlinks) = calculate_folder_size(&path.to_string_lossy());
                    total_size_with_hardlinks += sub_with_hardlinks;
                    total_size_without_hardlinks += sub_without_hardlinks;
                } else if path.is_file() {
                    // 获取文件大小
                    if let Ok(metadata) = fs::metadata(&path) {
                        let file_size = metadata.file_size();
                        total_size_with_hardlinks += file_size;

                        // 检查文件是否是硬链接
                        if !is_hard_link(&path) {
                            total_size_without_hardlinks += file_size;
                        }
                    }
                }
            }
        }
    }

    (total_size_with_hardlinks, total_size_without_hardlinks)
}

/// 检查文件是否是硬链接
fn is_hard_link(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        // 在 Windows 上，硬链接的文件具有多个硬链接计数
        metadata.number_of_links() > Some(1)
    } else {
        false
    }
}