const std = @import("std");
const fmt = std.fmt;
const io = std.io;
const mem = std.mem;
const warn = std.debug.warn;

fn hex2int(hexChars: []u8) !u128 {
    var hexSlice = hexChars[0..];
    if (mem.eql(u8, hexSlice[0..2], "0x")) {
        hexSlice = hexChars[2..];
    }

    // NOTE: If not for the fact that we need to ignore dashes ('-') in the
    //       input, we could replace this logic with a call to fmt.parseInt()
    var val: u128 = 0;
    for (hexSlice) |c| {
        if (c == '-') {
            continue;
        }
        var charVal = try fmt.charToDigit(c, 16);
        val = (val * 16) + charVal;
    }

    return val;
}

fn processInput(outstream: *io.OutStream(std.os.WriteError), input: []u8) void {
    if (hex2int(input)) |hexValue| {
        if (outstream.print("{}\n", hexValue)) {} else |err| {
            warn("Failed to write to stdout");
        }
    } else |parseErr| {
        warn("Failed to parse hex value {}: {}\n", input, parseErr);
    }
}

pub fn main() !void {
    var arenaAlloc = std.heap.ArenaAllocator.init(std.heap.direct_allocator);
    defer arenaAlloc.deinit();
    const alloc = &arenaAlloc.allocator;

    const stdinFile = try io.getStdIn();
    const stdoutFile = try io.getStdOut();
    var stdin = stdinFile.inStream();
    var stdout = stdoutFile.outStream();

    var args = std.process.args();
    const program = args.next(alloc);

    var argsPresent = false;
    while (args.next(alloc)) |argErrorUnion| {
        if (argErrorUnion) |hexArg| {
            processInput(&stdout.stream, hexArg);
            argsPresent = true;
        } else |err| {
            warn("Error when attempting to read argument: {}\n", err);
            break;
        }
    }

    if (!std.os.isatty(stdinFile.handle)) {
        var buf = try std.Buffer.initSize(alloc, 1024);
        defer buf.deinit();
        while (io.readLineFrom(&stdin.stream, &buf)) |hexArg| {
            processInput(&stdout.stream, hexArg);
        } else |err| {
            if (err != error.EndOfStream) {
                warn("Error while reading from stdin: {}\n", err);
            }
        }
    } else if (!argsPresent) {
        warn("Usage: {} INPUT...\n\nInput can also be piped in via stdin and will be processed one line at a time.\n", program);
    }
}
