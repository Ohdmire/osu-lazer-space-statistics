#![feature(windows_by_handle)]

use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use walkdir::WalkDir;
use rayon::prelude::*;

fn main() {

    let target_folder = get_lazer_location();

    println!("读取的osu路径为: {}", target_folder);

    println!("正在统计文件大小...");

    // 统计文件夹大小（包含硬链接和不包含硬链接）
    let (total_size_with_hardlinks, total_size_without_hardlinks) = calculate_folder_size(&target_folder);

    // 打印最大的单位
    println!("统计总大小（包含硬链接）: {}", format_size(total_size_with_hardlinks));
    println!("实际总大小（排除硬链接）: {}", format_size(total_size_without_hardlinks));

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("");

}

fn get_appdata_roaming() -> String {
    env::var("APPDATA").expect("$APPDATA不存在")
}

/// 获取lazer的路径
fn get_lazer_location() -> String {

    // 获取当前用户的 AppData\Roaming 路径
    let appdata_roaming = get_appdata_roaming();

    // 构建 storage.ini 文件的路径
    let storage_ini_path = Path::new(&appdata_roaming).join("osu").join("storage.ini");

    // 读取 storage.ini 文件内容
    let target_folder = match read_storage_ini(&storage_ini_path) {
        Ok(path) => path,
        Err(_) => {
            println!("读取 storage.ini 文件失败");
            println!("请手动输入文件夹例如D:\\osu\n留空则尝试默认文件夹");
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("");
            if input.trim().to_string().is_empty(){
                return Path::new(&appdata_roaming).join("osu").to_string_lossy().into_owned();
            }
            else { input.trim().to_string() }
        }
    };

    target_folder

}

/// 将字节大小格式化为最大的单位（如 1.2G、2.4M）
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

/// 文件元数据struct
#[derive(Debug)]
struct FileMetadata {
    size: u64,           // 文件大小
    is_hard_link: bool,  // 是否是硬链接
}
impl FileMetadata {
    /// 从路径获取文件元数据
    fn from_path(path: &Path) -> Option<Self> {
        if let Ok(metadata) = fs::metadata(path) {
            let size = metadata.len();
            let is_hard_link = metadata.number_of_links().unwrap_or(1) > 1;
            Some(Self { size, is_hard_link })
        } else {
            None
        }
    }
}

/// 统计文件夹大小（包含硬链接和不包含硬链接）
fn calculate_folder_size(folder: &str) -> (u64, u64) {
    // 使用 WalkDir 递归遍历文件夹
    let entries: Vec<_> = WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok()) // 过滤掉错误的条目
        .collect();

    // 使用 rayon 并行处理文件
    let results: Vec<_> = entries
        .par_iter() // 并行迭代
        .filter(|entry| entry.path().is_file()) // 判断文件
        .filter_map(|entry| FileMetadata::from_path(entry.path())) // 获取元数据
        .map(|metadata| (metadata.size, !metadata.is_hard_link)) // 映射为 (size, is_not_hard_link)
        .collect();

    // 汇总结果
    let (total_size_with_hardlinks, total_size_without_hardlinks) = results
        .into_par_iter() // 并行汇总
        .fold(
            || (0, 0), // 初始值
            |(acc_with, acc_without), (size, is_not_hardlink)| {
                (
                    acc_with + size,
                    acc_without + if is_not_hardlink { size } else { 0 },
                )
            },
        )
        .reduce(
            || (0, 0), // 初始值
            |(a_with, a_without), (b_with, b_without)| (a_with + b_with, a_without + b_without),
        );

    (total_size_with_hardlinks, total_size_without_hardlinks)
}