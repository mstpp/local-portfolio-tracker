#[cfg(test)]
pub mod fixtures {
    use crate::trade::Trade;
    use rstest::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[fixture]
    pub fn tickers() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "id,symbol,name\n\
             bitcoin,btc,Bitcoin\n\
             ethereum,eth,Ethereum\n\
             tether,usdt,Tether\n\
             binancecoin,bnb,BNB\n\
             cardano,ada,Cardano\n\
             usdt zero,usdt0,zero usdt0"
        )
        .unwrap();
        file.flush().unwrap();
    }

    #[fixture]
    pub fn transactions() -> Vec<Trade> {
        let csv_content = r#"created_at,pair,side,amount,price,fee
1704883200,BTC/USD,BUY,1.0,40000.00,7.50
1710460800,BTC/USD,BUY,3,20000.00,10.00"#;
        super::helpers::transactions_from(csv_content)
    }
}

#[cfg(test)]
pub mod helpers {
    use crate::currency::Currency;
    use crate::settings::Settings;
    use crate::trade::Trade;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::rc::Rc;
    use tempfile::TempDir;

    pub fn create_test_csv(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let file_path = dir.path().join(format!("{}.csv", name));
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }

    pub fn create_test_settings(base_path: PathBuf) -> Rc<Settings> {
        Rc::new(Settings {
            portfolio_dir: base_path,
            base_currency: Currency::new("USD").unwrap(),
        })
    }

    pub fn transactions_from(csv_content: &str) -> Vec<Trade> {
        let temp_dir = TempDir::new().unwrap();
        let file_name = "portfolio";
        create_test_csv(&temp_dir, file_name, csv_content);
        let settings = create_test_settings(temp_dir.path().to_path_buf());
        crate::trade::read_trades_from_csv(file_name, &settings).unwrap()
    }
}
