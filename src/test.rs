mod tests {
    use crate::*;
    use clap::Parser;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn reference_config(name: &str) -> Vec<u8> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("ref").join(name);
        fs::read(path).expect("reference config should be available")
    }

    fn temporary_directory() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("fbwf_creator_test_{suffix}"))
    }

    #[test]
    fn generated_configs_match_reference_files() {
        for (capacity, filename) in [
            (4096, "fbwf_4GB.cfg"),
            (8192, "fbwf_8GB.cfg"),
            (131_072, "fbwf_128GB.cfg"),
        ] {
            assert_eq!(build_config(capacity).unwrap(), reference_config(filename));
        }
    }

    #[test]
    fn boundaries_and_checksum_are_correct() {
        for capacity in [128, 131_072] {
            let config = build_config(capacity).unwrap();
            let units = (capacity / 16) as u16;
            assert_eq!(&config[0x1B..=0x1C], &units.to_le_bytes());
            assert_eq!(config.len(), CONFIG_SIZE);
            assert_eq!(config[0x00].wrapping_add(config[0x1C]), 0x24);
            assert_eq!(config[0x03].wrapping_add(config[0x1B]), 0xF8);
        }
    }

    #[test]
    fn invalid_capacities_are_rejected() {
        for capacity in [127, 129, 1000, 131_073] {
            assert!(build_config(capacity).is_err(), "{capacity} should fail");
        }
    }

    #[test]
    fn cli_rejects_non_numeric_and_negative_capacity() {
        assert!(Cli::try_parse_from(["FbwfCreator", "abc", "out.cfg"]).is_err());
        assert!(Cli::try_parse_from(["FbwfCreator", "-1", "out.cfg"]).is_err());
    }

    #[test]
    fn template_and_volume_path_are_present() {
        let config = build_config(4096).unwrap();
        assert_eq!(&config[0x01..=0x02], &[0xF9, 0x23]);
        assert_eq!(config[0x04], 0x02);
        assert_eq!(config[0x08], 0x01);
        assert_eq!(config[0x10], 0x02);
        assert_eq!(config[0x20], 0x01);
        assert_eq!(config[0x28], 0x02);

        let path_bytes: Vec<u8> = VOLUME_PATH
            .encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(&config[0x2C..0x2C + path_bytes.len()], path_bytes);
        assert!(
            config[0x2C + path_bytes.len()..]
                .iter()
                .all(|&byte| byte == 0)
        );
    }

    #[test]
    fn writing_requires_force_to_overwrite() {
        let directory = temporary_directory();
        fs::create_dir(&directory).unwrap();
        let output = directory.join("custom.cfg");
        let original = vec![0xAA; CONFIG_SIZE];
        fs::write(&output, &original).unwrap();

        let config = build_config(4096).unwrap();
        assert!(write_config(&output, &config, false).is_err());
        assert_eq!(fs::read(&output).unwrap(), original);
        write_config(&output, &config, true).unwrap();
        assert_eq!(fs::read(&output).unwrap(), config);

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn writing_requires_existing_parent_directory() {
        let output = temporary_directory().join("custom.cfg");
        let config = build_config(4096).unwrap();
        assert!(write_config(&output, &config, false).is_err());
    }
}
