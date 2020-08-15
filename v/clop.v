import clipboard
import os

fn print_usage(program_name string) {
    println("Copy text to & from the system clipboard from the terminal")
    println("")
    println("Usage: $program_name [OPERATION] [INPUT]...")
    println("")
    println("Operations:")
    println("    help  - Show this help text")
    println("    in    - Copy the remainder of the program arguments to the clipboard")
    println("    stdin - Copy the complete contents of stdin to the clipboard")
    println("    out   - Print the contents of the clipboard to stdout")
}

fn main() {
    program_name := os.args[0]
    if os.args.len < 2 {
        print_usage(program_name)
        return
    }

    operation := os.args[1]
    cb := clipboard.new()

    if operation == "help" {
        print_usage(program_name)

    } else if operation == "in" {
        mut remaining_args := ""
        for _, arg in os.args[2..] {
            if remaining_args.len != 0 {
                remaining_args += ' '
            }
            remaining_args += arg
        }
        cb.copy(remaining_args)
        println("'$remaining_args' copied to the clipboard")

    } else if operation == "stdin" {
        // TODO: We'd like to just use 'in' for this, but we need access to istty (or some equivalent function) to decide where to get input from (args or stdin)
        mut all_text := ""
        mut lines := 0
        for {
            line := os.get_raw_line()
            if line.len <= 0 {
                break
            }
            all_text += line
            lines++
        }
        cb.copy(all_text)
        println("$lines lines copied to the clipboard")

    } else if operation == "out" {
        println(cb.paste())

    } else {
        println("Invalid operation '$operation': options are 'in' or 'out'. Exiting...")
        return
    }
}
