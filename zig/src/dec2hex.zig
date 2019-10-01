const std = @import("std");
const fmt = std.fmt;
const io = std.io;
const warn = std.debug.warn;

fn processInput(outstream: *io.OutStream(std.os.WriteError), input: []u8) void {
    if (fmt.parseInt(u128, input, 10)) |decVal| {
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

    const stdinFile = try io.getStdIn();
    const stdoutFile = try io.getStdOut();
    var stdin = stdinFile.inStream();
    var stdout = stdoutFile.outStream();

    var args = std.process.args();
    const program = args.next(alloc);

    var argsPresent = false;
    while (args.next(alloc)) |argErrorUnion| {
        if (argErrorUnion) |decArg| {
            processInput(&stdout.stream, decArg);
            argsPresent = true;
        } else |err| {
            warn("Error when attempting to read argument: {}\n", err);
            break;
        }
    }

    if (!std.os.isatty(stdinFile.handle)) {
        var buf = try std.Buffer.initSize(alloc, 1024);
        defer buf.deinit();
        while (io.readLineFrom(&stdin.stream, &buf)) |decArg| {
            processInput(&stdout.stream, decArg);
        } else |err| {
            if (err != error.EndOfStream) {
                warn("Error while reading from stdin: {}\n", err);
            }
        }
    } else if (!argsPresent) {
        warn("Usage: {} INPUT...\n\nInput can also be piped in via stdin and will be processed one line at a time.\n", program);
    }
}
