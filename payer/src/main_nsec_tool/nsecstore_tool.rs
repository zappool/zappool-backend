///! Utility tool implementation: tool to create or check an encrypted secret nsec file.
use bech32::{FromBase32, ToBase32, decode, encode};
use secp256k1::Secp256k1;
use seedstore::{KeyStore, KeyStoreCreator, Options, SeedStore};
use std::fs;

const DEFAULT_FILE_NAME: &str = "secret.nsec";

#[derive(PartialEq)]
enum Mode {
    /// Create new file
    Set,
    /// Check existing file
    Check,
    /// Help only
    Help,
}

struct Config {
    mode: Mode,
    filename: String,
    program_name: String,
    allow_weak_password: bool,
}

/// Utility tool implementation: tool to create or check an encrypted secret nsec file.
pub struct NsecStoreTool {
    config: Config,
}

impl Config {
    fn default() -> Self {
        Self {
            mode: Mode::Check,
            filename: DEFAULT_FILE_NAME.to_owned(),
            program_name: "tool".to_owned(),
            allow_weak_password: false,
        }
    }
}

impl ToString for Config {
    fn to_string(&self) -> std::string::String {
        let mut s = String::with_capacity(200);
        s += &format!("[{}]:  ", self.program_name);
        s += "Mode: ";
        s += match self.mode {
            Mode::Check => "Check only",
            Mode::Set => "Set",
            Mode::Help => "Help only",
        };
        s += &format!("   File: {}", self.filename);
        s
    }
}

impl NsecStoreTool {
    pub fn new(args: &Vec<String>) -> Result<Self, String> {
        // Process cmd line arguments
        let config = Self::process_args(args)?;

        Ok(Self::new_from_config(config))
    }

    fn new_from_config(config: Config) -> Self {
        Self { config }
    }

    pub fn print_usage(progname: &Option<&String>) {
        let default_progname = "tool".to_owned();
        let progname = progname.unwrap_or(&default_progname);
        println!("{}:  Set or check secret nsec file", progname);
        println!("");
        println!("{}  [--help] [--set] [--file <file>] [--weakpw]", progname);
        println!(
            "  --set:         If specified, mnemominc is prompted for, and secret is saved. Secret file must not exist."
        );
        println!("                 Default is to only check secret file, and print the npub");
        println!(
            "  --file <file>  Secret file to use, default is {}",
            DEFAULT_FILE_NAME
        );
        println!("  --weakpw       Allow weak encryption password");
        println!("  --help         Print usage (this)");
        println!("");
    }

    fn process_args(args: &Vec<String>) -> Result<Config, String> {
        let mut config = Config::default();
        let len = args.len();
        if len < 1 {
            return Err("Internal arg error, progname missing".to_owned());
        }
        debug_assert!(len >= 1);
        config.program_name = args[0].clone();
        let mut i = 1;
        while i < len {
            let a = &args[i];
            if *a == "--set" {
                config.mode = Mode::Set;
            } else if *a == "--file" {
                if i + 1 < len {
                    config.filename = args[i + 1].clone();
                    i += 1;
                } else {
                    return Err("--file requires a <file> argument".to_owned());
                }
            } else if *a == "--help" {
                config.mode = Mode::Help;
            } else if *a == "--weakpw" {
                config.allow_weak_password = true;
            } else {
                return Err(format!("Unknown argument {}", a));
            }
            i += 1;
        }

        Ok(config)
    }

    pub fn run(args: &Vec<String>) {
        match Self::new(&args) {
            Err(err) => {
                println!("Error processing arguments! {}", err);
                Self::print_usage(&args.get(0));
            }
            Ok(mut tool) => match tool.execute() {
                Err(err) => println!("ERROR: {}", err),
                Ok(_) => {
                    println!("Done.");
                }
            },
        }
    }

    /// Return npub (for testing)
    pub fn execute(&mut self) -> Result<String, String> {
        println!("{}", self.config.to_string());

        match self.config.mode {
            Mode::Set => self.do_set(),
            Mode::Check => self.do_check(),
            Mode::Help => {
                Self::print_usage(&Some(&self.config.program_name));
                Ok("".to_owned())
            }
        }
    }

    /// Perform Set file operation
    /// Return npub (for testing)
    fn do_set(&mut self) -> Result<String, String> {
        let exists = fs::exists(&self.config.filename).unwrap_or(true);
        if exists {
            return Err(format!(
                "File already exists, won't overwrite, aborting {}",
                self.config.filename
            ));
        }

        let nsec_str = self.read_nsec()?;
        let nsec_decoded =
            decode(&nsec_str).map_err(|e| format!("Invalid nsec {}", e.to_string()))?;
        if nsec_decoded.0 != "nsec" {
            return Err(format!("Unexcpeted HRP {}", nsec_decoded.0).into());
        }
        let nsec = Vec::<u8>::from_base32(&nsec_decoded.1)
            .map_err(|e| format!("Invalid bech32 {}", e.to_string()))?;
        let nsec: [u8; 32] = nsec
            .try_into()
            .map_err(|_e| format!("Invalid bech32 length"))?;
        println!("Nsec entered, seems OK");

        let password = self.read_password()?;
        if !self.config.allow_weak_password {
            let _res = SeedStore::validate_password(&password)?;
        }

        let keystore = KeyStoreCreator::new_from_data(&nsec)
            .map_err(|e| format!("Could not encrypt secret, {}", e))?;

        let npub = self.print_info(&keystore)?;

        let options = if self.config.allow_weak_password {
            Some(Options::new().allow_weak_password())
        } else {
            None
        };
        let _res =
            KeyStoreCreator::write_to_file(&keystore, &self.config.filename, &password, options)
                .map_err(|e| format!("Could not write secret file, {}", e))?;

        println!("Nsec written to encrypted file: {}", self.config.filename);

        Ok(npub)
    }

    fn read_no_echo(&mut self, item_name: &str, prompt: &str) -> Result<String, String> {
        let result = rpassword::prompt_password(prompt)
            .map_err(|e| format!("Error reading {}, {}", item_name, e.to_string()))?;
        Ok(result)
    }

    fn read_password(&mut self) -> Result<String, String> {
        let password1 = self.read_no_echo("password", "Enter the encryption password: ")?;
        let password2 = self.read_no_echo("password", "Repeat the encryption password: ")?;
        if password1 != password2 {
            return Err("The two passwords don't match".to_owned());
        }
        println!("Passwords entered, match OK");
        debug_assert_eq!(password1, password2);
        Ok(password1)
    }

    fn read_nsec(&mut self) -> Result<String, String> {
        let nsec = self.read_no_echo("nsec", "Enter the nsec (input is hidden): ")?;
        Ok(nsec)
    }

    /// Print out info from the seedstore
    /// Return npub (for testing)
    fn print_info(&self, keystore: &KeyStore) -> Result<String, String> {
        let secp = Secp256k1::new();
        let pubkey = keystore
            .get_secret_private_key()
            .map_err(|e| e.to_string())?
            .x_only_public_key(&secp)
            .0
            .serialize();
        let npub = encode("npub", pubkey.to_base32(), bech32::Variant::Bech32)
            .map_err(|e| e.to_string())?;
        println!("npub: {}", npub);
        println!("");

        Ok(npub)
    }

    /// Return XPub (for testing)
    fn do_check(&mut self) -> Result<String, String> {
        let exists = fs::exists(&self.config.filename).unwrap_or(false);
        if !exists {
            return Err(format!(
                "Could not find secret file {}",
                self.config.filename
            ));
        }

        let password = self.read_password()?;

        let keystore = KeyStore::new_from_encrypted_file(&self.config.filename, &password)
            .map_err(|e| format!("Could not read secret file, {}", e))?;

        println!("");
        println!(
            "Nsec has been read from secret file {}",
            self.config.filename
        );

        let npub = self.print_info(&keystore)?;

        Ok(npub)
    }
}
