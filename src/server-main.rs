extern crate mize;

static HELP_MESSAGE: &str = "\
Usage:
    mize-server [options] [command] [options for command]

Available Commands:
    run                starts the server
    help               prints this help message
    version            prints the version
    import             imports a file as a item of type file into the server

Available options
    -h --help          prints this help message
    -v --version       prints the version
    -f --folder        the mize-server-folder   
";

//some flags e.g: "--file /tmp" can require that the next argument belongs to them instead of being
//the command
static FLAGS_WITH_ARGUMENTS: [&str; 2] = ["--folder", "-f"];

static AVAILABLE_COMMANDS: [&str; 6] = ["run", "help", "version", "import-js", "import-render", "import"];

static AVAILABLE_FLAGS: [&str; 4] = ["--version", "--help", "--folder", "-f"];
//static AVAILABLE_ONE_LETTER_FLAGS: [&str; 2] = ["v", "h"];

static VERSION_MESSAGE: &str = "\
Version: 0
";


fn main() {
    /*
     * WHAT IT DOES
     * - get the arguments passed to the programm
     *
     * - sort out the flags for the main programm and the args for the comand
     *     - in "mize -h -u --lol server --file=/tmp --folder /home" the flags for the main programm would be ["-h","-u","--lol"]
     *     - and the args for the command would be ["server", "--file=/tmp --folder /home"]
     *
     * - get the command to "run"
     *     - in "mize -h -u --lol server --file=/tmp" it would be "server"
     *
     * - "run" the command and pass the options and subcommands to it
     *
     */

    //### get the arguments passed to the programm
    let args: Vec<String> = std::env::args().collect();

    //### sort out the flags for the main program and the args for the command
    let mut main_flags:Vec<String> = Vec::new();
    let mut command= String::from("");
    let mut command_args = Vec::new();

    for i in 1..args.len() {
        if args[i].chars().next().unwrap() == '-' {
            main_flags.push(args[i].clone());
            // i != 0 bcs what if the programm name is in FLAGS_WITH_ARGUMENTS for some reason?
        } else if i != 0 && FLAGS_WITH_ARGUMENTS.contains(&&args[i -1][..]) {
            main_flags.push(args[i].clone());

    //#### get the command to "run"
        } else if AVAILABLE_COMMANDS.contains(&&args[i][..]){
            command = args[i].clone();
            for a in i..args.len(){
                command_args.push(args[a].clone());
            }
            break;
        } else {
            println!("Command \"{}\" not found!!\n", args[i]);
            println!("{}", HELP_MESSAGE);
            std::process::exit(1);
        }
    }

    //#### "run" the command and pass the options and subcommands to it
    match &command[..] {
        "run" => mize::server::run_server(args),
        "import" => mize::server::server_utils::import(args),
        "help" => println!("{}", HELP_MESSAGE),
        "version" => println!("{}", VERSION_MESSAGE),
        _ => {println!("Command not found!!\n"); println!("{}", HELP_MESSAGE);},
    }

}

