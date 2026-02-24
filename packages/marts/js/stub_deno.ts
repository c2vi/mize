export class Mize {
  async get_config(key: string): Promise<any> {
    return "hiiiiiii";
  }
  async add_opts(opts: any): Promise<void> {}
  async add_part(part: any): Promise<void> {}
  async report_err(err: MizeError) {}
}

type MizeError = string;

export class Cli {
  constructor(mize: Mize) {}
  async sub_command(name: string, cmd: any): Promise<void> {}
}
