pub fn handle_echo(args: Vec<String>) -> Result<String, String> {
    Ok(format!("{}", args.join(" ")))
}
