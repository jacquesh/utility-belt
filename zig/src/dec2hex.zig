const std = @import("std");
const buffer = std.Buffer;
const fmt = std.fmt;
const io = std.io; // TODO remove? No warning?
const mem = std.mem;
const os = std.os;
const warn = std.debug.warn;

fn processInput(outstream: *io.OutStream(os.WriteError), input: []u8) void {
    if (fmt.parseInt(u128, input, 10)) |decVal| {
        // TODO: This (probably? it does in hex2dec) fails on Windows with "error: Unexpected":
        if (outstream.print("0x{X}\n", decVal)) {} else |err| {
            warn("Failed to write to stdout: {}\n", err);
        }
    } else |parseErr| {
        warn("Failed to parse decimal value {}: {}\n", input, parseErr);
    }
}

pub fn main() !void {
    var arenaAlloc = std.heap.ArenaAllocator.init(std.heap.direct_allocator);
    defer arenaAlloc.deinit();
    const alloc = &arenaAlloc.allocator;

    var stdinFile = try std.io.getStdIn();
    const stdoutFile = try std.io.getStdOut();
    var stdin = stdinFile.inStream().stream;
    var stdout = stdoutFile.outStream().stream;
    warn("Stdin types: file={}, stream={}\n", @typeName(@typeOf(stdinFile)), @typeName(@typeOf(&stdin)));
    warn("Stdot types: file={}, stream={}\n", @typeName(@typeOf(stdoutFile)), @typeName(@typeOf(&stdout)));

    var args = std.process.args();
    const program = args.next(alloc);

    var argsPresent = false;
    while (args.next(alloc)) |argErrorUnion| {
        if (argErrorUnion) |decArg| {
            processInput(&stdout, decArg);
            argsPresent = true;
        } else |err| {
            warn("Error when attempting to read argument: {}\n", err);
            break;
        }
    }

    if (!std.os.isatty(stdinFile.handle)) {
        warn("Stdin is a TTY\n");
        var buf = try buffer.initSize(alloc, 128);
        //defer buf.deinit();
        while (io.readLineFrom(&stdin, &buf)) |decArg| {
            warn("Read input line: {}\n", decArg);
            processInput(&stdout, decArg);
        } else |err| {
            warn("Error while reading from stdin: {}\n", err);
        }
    } else if (!argsPresent) {
        warn("Usage: {} INPUT...\n", program);
    }
}
