#![feature(windows_by_handle)]

use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use walkdir::WalkDir;
use rayon::prelude::*; // 引入 rayon 库

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
/// 统计文件夹大小（包含硬链接和不包含硬链接）
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
        .filter(|entry| entry.path().is_file()) // 只处理文件
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

fn criterion_benchmark(c: &mut Criterion) {
    let folder = "D:\\osu"; // 替换为你要测试的文件夹路径

    c.bench_function("calculate_folder_size", |b| {
        b.iter(|| calculate_folder_size(folder))
    });
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);