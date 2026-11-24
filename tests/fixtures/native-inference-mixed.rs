use std::cell::Cell;
use std::ffi::OsString;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

const WIDTH: usize = 2;
enum Code {
    Start = 7,
    End = 11,
}

struct Guard(Rc<Cell<usize>>);
impl Drop for Guard {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn before() -> usize {
    let f = || Code::Start as usize;
    f()
}

fn arguments(jobs: usize) -> Vec<OsString> {
    let mut args = Vec::new();
    args.extend([OsString::from("base")]);
    args.extend(["flag".into(), format!("jobs={jobs}").into()]);
    args
}

fn after(n: usize) -> usize {
    let captured = Code::End as usize;
    let f = move |x: usize| x + captured;
    f(n)
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let drops = Rc::new(Cell::new(0));
    {
        let _outer = Guard(drops.clone());
        let mut args = arguments(before());
        assert_eq!(args[2], OsString::from("jobs=7"));
        let capacity = args[0].capacity();
        let address = args[0].as_encoded_bytes().as_ptr();
        assert!(catch_unwind(AssertUnwindSafe(|| args[0].reserve(usize::MAX))).is_err(), "overflow must unwind");
        assert_eq!(args[0].capacity(), capacity);
        assert_eq!(args[0].as_encoded_bytes().as_ptr(), address);
        assert_eq!(args[0], OsString::from("base"));
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let _inner = Guard(drops.clone());
                panic!("drop witness");
            }))
            .is_err(),
            "guard panic must unwind"
        );
        assert_eq!(drops.get(), 1);
        let values: [usize; WIDTH] = [before(), after(2)];
        assert_eq!(values[0] + values[1], 20);
    }
    assert_eq!(drops.get(), 2);
    println!(
        "mixed inference, arenas, overflow and drop: {} / {}",
        drops.get(),
        option_env!("LUSTR_NATIVE_INFERENCE_LABEL").unwrap_or("unset")
    );
}
