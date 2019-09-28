const zigbuild = @import("std").build;

fn createExeStep(b: *zigbuild.Builder, name: []const u8, rootFilePath: []const u8) *zigbuild.LibExeObjStep {
    const build_mode = b.standardReleaseOptions();
    const exe = b.addExecutable(name, rootFilePath);
    exe.setBuildMode(build_mode);
    exe.setOutputDir("bin");
    return exe;
}

pub fn build(b: *zigbuild.Builder) void {
    const all_step = b.step("all", "Build all executables");

    all_step.dependOn(&createExeStep(b, "hex2dec", "src/hex2dec.zig").step);
    all_step.dependOn(&createExeStep(b, "dec2hex", "src/dec2hex.zig").step);

    b.default_step.dependOn(all_step);
}
