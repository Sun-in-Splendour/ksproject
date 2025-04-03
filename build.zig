const std = @import("std");

pub fn build(b: *std.Build) !void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    var target_path: []const u8 = "target/debug/";
    var libkslang_path: []const u8 = "target/debug/libkslang.a";
    const fs = std.fs.cwd();
    fs.access("target/debug/", .{}) catch |err| {
        switch (err) {
            std.posix.AccessError.FileNotFound => {
                try fs.access("target/release/", .{});
                target_path = "target/release/";
                libkslang_path = "target/release/libkslang.a";
            },
            else => return err,
        }
    };

    const ksc_source = b.addExecutable(.{
        .name = "ksc_source",
        .target = target,
        .optimize = optimize,
    });

    ksc_source.addIncludePath(b.path("include"));
    ksc_source.addLibraryPath(b.path(target_path));
    ksc_source.addCSourceFile(.{ .file = b.path("tests/ksc_source.cpp"), .flags = &.{} });

    ksc_source.linkLibCpp();
    ksc_source.addObjectFile(b.path(libkslang_path));

    b.installArtifact(ksc_source);

    const run = b.addRunArtifact(ksc_source);
    run.step.dependOn(b.getInstallStep());

    if (b.args) |args| {
        run.addArgs(args);
    }

    const run_step = b.step("run", "run the test");
    run_step.dependOn(&run.step);
}
