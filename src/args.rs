#[derive(Debug)]
pub struct Args {
    pub name: Option<String>,
    pub name_bytes: Option<Vec<u8>>,
    pub iname: Option<String>,
    pub iname_bytes: Option<Vec<u8>>,
    pub type_filter: Option<char>, // 'f' or 'd'
}

pub fn parse_args(args: &[String]) -> Args {
    let mut opts = Args {
        name: None,
        name_bytes: None,
        iname: None,
        iname_bytes: None,
        type_filter: None,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-name" if i + 1 < args.len() => {
                let pattern = args[i + 1].clone();
                opts.name_bytes = Some(pattern.as_bytes().to_vec());
                opts.name = Some(pattern);
                i += 1;
            }
            "-iname" if i + 1 < args.len() => {
                let pattern = args[i + 1].clone();
                let pattern_lower = pattern.to_lowercase();
                opts.iname_bytes = Some(pattern_lower.as_bytes().to_vec());
                opts.iname = Some(pattern);
                i += 1;
            }
            "-type" if i + 1 < args.len() => {
                let t = args[i + 1].chars().next().unwrap_or('_');
                if t == 'f' || t == 'd' {
                    opts.type_filter = Some(t);
                } else {
                    eprintln!("find: invalid argument to '-type': {}", args[i + 1]);
                }
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    opts
}
