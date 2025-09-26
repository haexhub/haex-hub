// models.rs




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_pattern() {
        // Valid patterns
        assert!(validate_path_pattern("$PICTURE/**").is_ok());
        assert!(validate_path_pattern("$DOCUMENT/myfiles/*").is_ok());
        assert!(validate_path_pattern("$APPDATA/extensions/my-ext/**").is_ok());

        // Invalid patterns
        assert!(validate_path_pattern("").is_err());
        assert!(validate_path_pattern("$INVALID/test").is_err());
        assert!(validate_path_pattern("$PICTURE/../secret").is_err());
        assert!(validate_path_pattern("relative/path").is_err());
    }

    #[test]
    fn test_filesystem_permissions() {
        let mut perms = FilesystemPermissions::new();
        perms.add_read("$PICTURE/**");
        perms.add_write("$APPDATA/my-ext/**");

        assert!(perms.validate().is_ok());
        assert_eq!(perms.read.as_ref().unwrap().len(), 1);
        assert_eq!(perms.write.as_ref().unwrap().len(), 1);
    }
}