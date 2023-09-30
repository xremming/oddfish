use mx::{table, Table, Value};

#[derive(Debug)]
enum CallError {
    NotEnoughArguments(usize, usize),
    InvalidArgumentType(usize, usize),
    // TODO: support fallible functions
    // FunctionReturnedError(Option<E>),
}

type CallResult = Result<Value, CallError>;

trait NativeFunction<A, const N: usize, R, const M: bool, const F: bool = false> {
    fn into_callable(self) -> Callable;
}

impl<FN: Fn() -> RET + 'static, RET: Into<Value>> NativeFunction<(), 0, RET, false> for FN {
    fn into_callable(self) -> Callable {
        Callable::Function(Box::new(move |_args: Table| self().into()))
    }
}

impl<FN: Fn() -> Option<RET> + 'static, RET: Into<Value>> NativeFunction<(), 0, RET, false, true>
    for FN
{
    // TODO: change the return value from Value to ReturnValue enum
    fn into_callable(self) -> Callable {
        Callable::Function(Box::new(move |_args: Table| match self() {
            Some(v) => v.into(),
            None => Value::nil(),
        }))
    }
}

impl<FN: Fn(A1) -> RET + 'static, A1: TryFrom<Value>, RET: Into<Value>>
    NativeFunction<(A1), 1, RET, false> for FN
{
    fn into_callable(self) -> Callable {
        Callable::Function(Box::new(move |args: Table| {
            let arg0 = args
                .get(0)
                .ok_or(CallError::NotEnoughArguments(0, 1))
                .unwrap()
                .clone()
                .get_value()
                .ok_or(CallError::InvalidArgumentType(0, 1))
                .unwrap();

            self(arg0).into()
        }))
    }
}

impl<FN: Fn(A1, A2) -> RET + 'static, A1: TryFrom<Value>, A2: TryFrom<Value>, RET: Into<Value>>
    NativeFunction<(A1, A2), 2, RET, false> for FN
{
    fn into_callable(self) -> Callable {
        Callable::Function(Box::new(move |args: Table| {
            let arg0 = args[0].clone().get_value().unwrap();
            let arg1 = args[1].clone().get_value().unwrap();
            self(arg0, arg1).into()
        }))
    }
}

impl<
        FN: Fn(A1, A2, A3) -> RET + 'static,
        A1: TryFrom<Value>,
        A2: TryFrom<Value>,
        A3: TryFrom<Value>,
        RET: Into<Value>,
    > NativeFunction<(A1, A2, A3), 3, RET, false> for FN
{
    fn into_callable(self) -> Callable {
        Callable::Function(Box::new(move |args: Table| {
            let arg0 = args[0].clone().get_value().unwrap();
            let arg1 = args[1].clone().get_value().unwrap();
            let arg2 = args[2].clone().get_value().unwrap();
            self(arg0, arg1, arg2).into()
        }))
    }
}

impl<
        FN: Fn(&mut Table, A1, A2, A3) -> RET + 'static,
        A1: TryFrom<Value>,
        A2: TryFrom<Value>,
        A3: TryFrom<Value>,
        RET: Into<Value>,
    > NativeFunction<(A1, A2, A3), 3, RET, true> for FN
{
    fn into_callable(self) -> Callable {
        Callable::Method(Box::new(move |self_: &mut Table, args: Table| {
            let arg0 = args[0].clone().get_value().unwrap();
            let arg1 = args[1].clone().get_value().unwrap();
            let arg2 = args[2].clone().get_value().unwrap();
            self(self_, arg0, arg1, arg2).into()
        }))
    }
}

enum ReturnValue {
    Value(Value),
    None,
}

type FnAny = dyn Fn(Table) -> Value;
type MethodAny = dyn Fn(&mut Table, Table) -> Value;

enum Callable {
    Function(Box<FnAny>),
    Method(Box<MethodAny>),
    // Table(Table),
}

impl Callable {
    fn new<A, const N: usize, R, const M: bool>(f: impl NativeFunction<A, N, R, M>) -> Self {
        f.into_callable()
    }

    fn function(&self, args: Table) -> Option<Value> {
        match self {
            Callable::Function(f) => Some(f(args)),
            Callable::Method(f) => None,
        }
    }

    fn method(&self, self_: &mut Table, args: Table) -> Option<Value> {
        match self {
            Callable::Function(f) => None,
            Callable::Method(f) => Some(f(self_, args)),
        }
    }
}

fn main() {
    let f = Callable::new(|num: f32| {
        println!("hello {}", num);
    });

    let res = f.function(table![420]);
    println!("{:?}", res);

    let m = Callable::new(|self_: &mut Table, num: f32, _: (), _: ()| {
        println!("method {}", num);
        self_.set("x", num);
    });

    let mut data = table! { "x" => 0 };
    let res = m.method(&mut data, table![69, (), ()]);
    println!("res={:?} data={:?}", res, data);

    let v = Value::new(true);
    println!("{:?}", v);

    let globals = table! {
        "abba" => true,
        "baab" => "hello world",
        "nil" => (),
        "xs" => table![1, 2, 3, 4, 5],
        // "abs" => Callable::new(|v: f64| -> f64 {v.abs()}),
    };

    let v = Value::new(globals.clone());
    println!("{:?}", v);
    println!("{:?}", v.get_value().unwrap_or(0));

    println!("{:?}", globals);
    println!("{:?}", globals["abba"]);
    println!("{:?}", globals["nil"]);
    println!("{:?}", globals.get("nonexistent"));

    let list = table![true, false, 12, (), "never seen",];
    println!("{:?}", list);

    for v in list.iter_list() {
        println!("{:?}", v);
    }

    for (k, v) in globals {
        println!("{:?} => {:?}", k, v);
    }
}
