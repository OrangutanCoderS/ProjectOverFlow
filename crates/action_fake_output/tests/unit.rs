use action_fake_output::controller::PlatformFakeOutput;
use action_fake_output::FakeOutputController;

#[test]
fn test_fake_stdout() {
    let pid = std::process::id() as i32;
    assert!(PlatformFakeOutput::inject_stdout(pid, "HelloFake").is_ok());
}

#[test]
fn test_fake_file_read() {
    let tmpfile = "/tmp/fake_test.txt";
    assert!(PlatformFakeOutput::fake_file_read(tmpfile, "FakeData").is_ok());
}