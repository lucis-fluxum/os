use crate::qemu;
use crate::serial_println;

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    qemu::exit(qemu::ExitCode::Success);
}
