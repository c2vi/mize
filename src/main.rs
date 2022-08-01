
pub mod server;

static HELP_MESSAGE: &str = "\
Usage:
    mize [options] [command] [options for command]

Available Commands:
    server          starts the server
    help            prints this help message
    version         prints the version

Available options
    -h --help       prints this help message
    -v --version    prints the version
";

//some flags e.g: "--file /tmp" can require that the next argument belongs to them instead of being
//the command
static FLAGS_WITH_ARGUMENTS: [&str: 0] = [];

static AVAILABLE_COMMANDS: [&str: 1] = [server];

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
    let main_flags:Vec<String> = [].to_vec();

    for arg in &args[1..] {
        //if arg
        println!("{}", arg.chars().next().unwrap())
    }

    //#### get the command to "run"
    
    //#### "run" the command and pass the options and subcommands to it

}

