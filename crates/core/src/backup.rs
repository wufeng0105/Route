use std::fs;
use std::path::Path;

/// 创建配置文件备份
///
/// 将原文件复制为 `<filename>.backup.<timestamp>`
/// 返回备份文件路径
pub fn create_backup(file_path: &Path) -> std::io::Result<std::path::PathBuf> {
    let timestamp = chrono_timestamp();
    let backup_name = format!(
        "{}.backup.{}",
        file_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        timestamp
    );
    let backup_path = file_path.with_file_name(backup_name);

    fs::copy(file_path, &backup_path)?;

    // 在非 Windows 平台上保留原文件权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(file_path)?;
        fs::set_permissions(&backup_path, metadata.permissions())?;
    }

    Ok(backup_path)
}

/// 回滚：将备份文件恢复为原文件
///
/// 1. 删除可能损坏的原文件
/// 2. 将备份文件重命名为原文件名
pub fn rollback(backup_path: &Path, original_path: &Path) -> std::io::Result<()> {
    // 删除可能写入失败的原文件
    if original_path.exists() {
        fs::remove_file(original_path)?;
    }

    // 将备份重命名为原文件
    fs::rename(backup_path, original_path)?;

    // 恢复权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(original_path)?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o644);
        fs::set_permissions(original_path, perms)?;
    }

    Ok(())
}

/// 生成时间戳字符串 (YYYYMMDD_HHMMSS)
fn chrono_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // 简单的时间戳转换（不依赖 chrono crate）
    let secs = now % 60;
    let mins = (now / 60) % 60;
    let hours = (now / 3600) % 24;

    // 计算日期（从 1970-01-01 开始）
    let days = now / 86400;
    let (year, month, day) = days_to_date(days as i64);

    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        year, month, day, hours, mins, secs
    )
}

/// 将 Unix 天数转换为年月日
fn days_to_date(days: i64) -> (i64, u32, u32) {
    // 从 1970 年开始计算
    let mut remaining_days = days;
    let mut year = 1970i64;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let months_days = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for &dim in &months_days {
        if remaining_days < dim {
            break;
        }
        remaining_days -= dim;
        month += 1;
    }

    let day = remaining_days as u32 + 1;
    (year, month, day)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_create_backup() {
        let dir = std::env::temp_dir();
        let file_path = dir.join("test_backup_create.txt");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();
        drop(file);

        let backup_path = create_backup(&file_path).unwrap();
        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains(".backup."));

        // 清理
        let _ = fs::remove_file(&file_path);
        let _ = fs::remove_file(&backup_path);
    }

    #[test]
    fn test_rollback() {
        let dir = std::env::temp_dir();
        let file_path = dir.join("test_backup_rollback.txt");
        let backup_path = dir.join("test_backup_rollback.txt.backup.20260724_120000");

        // 创建原文件
        fs::write(&file_path, "original content").unwrap();
        // 创建备份
        fs::write(&backup_path, "backup content").unwrap();

        // 破坏原文件
        fs::write(&file_path, "corrupted").unwrap();

        // 回滚
        rollback(&backup_path, &file_path).unwrap();

        // 验证恢复
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "backup content");
        assert!(!backup_path.exists());

        // 清理
        let _ = fs::remove_file(&file_path);
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2023));
    }

    #[test]
    fn test_days_to_date() {
        // 1970-01-01 = day 0
        assert_eq!(days_to_date(0), (1970, 1, 1));
        // 1970-01-02 = day 1
        assert_eq!(days_to_date(1), (1970, 1, 2));
        // 1971-01-01 = day 365
        assert_eq!(days_to_date(365), (1971, 1, 1));
    }
}
