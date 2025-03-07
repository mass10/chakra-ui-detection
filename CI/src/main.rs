fn execute_command(command: &[&str]) -> std::process::Output {
    std::process::Command::new(command[0])
        .args(&command[1..])
        .output()
        .expect("failed to execute process")
}

fn main() {
    println!("### START ###");

    execute_command(&["find", "src/components/chakra", "-type", "f"]);

    println!("--- END ---");
}
