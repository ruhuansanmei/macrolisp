#![no_std]

pub mod prelude;

// TODO documentation
// TODO attributes
// TODO MACROS
#[macro_export] macro_rules! lisp {
    // empty
    () => (());
    (()) => (());

    // special forms
    ((ns $($ns_form:tt)*) $($body:tt)*) => {
        $(lisp!($ns_form);)*
        $(lisp!($body);)*
    };
    ((extern [$($krate:tt)*])) => {
        $(lisp!(@krate $krate);)*
    };
    ((use [$($uze:tt)*])) => {
        $(lisp!(@uze $uze);)*
    };
    ((lambda [$($argn:ident)*] $($body:tt)*)) => {
        // regular lambda
        |$($argn),*| { $(lisp!($body));* }
    };
    ((lambda ([$(($argn:ident $argt:ty))*] $ret:ty) $($body:tt)*)) => {
        // regular lambda
        |$($argn:$argt),*| -> $ret { $(lisp!($body));* }
    };
    ((lambda $s:ident ([$(($argn:ident $argt:ty))*] $ret:ty) $($body:tt)*)) => {{
        // recursive lambda
        // $s MUST NOT be "self"
        // recurse by calling ($s ...)
        // FIXME recursive lambdas can't capture variables

        fn $s($($argn: $argt),*) -> $ret { $(lisp!($body));* }
        $s
    }};
    ((defn $name:ident () $($body:tt)*)) => {
        fn $name() {
            $(lisp!($body));*
        }
    };
    ((defn $name:ident ([($selph:ident) $(($argn:ident $argt:ty))*] $ret:ty) $($body:tt)*)) => {
        fn $name($selph $(, $argn:$argt)*) -> $ret {
            $(lisp!($body));*
        }
    };
    ((defn $name:ident ([(&$selph:ident) $(($argn:ident $argt:ty))*] $ret:ty) $($body:tt)*)) => {
        fn $name(&$selph $(, $argn:$argt)*) -> $ret {
            $(lisp!($body));*
        }
    };
    ((defn $name:ident ([(&mut $selph:ident) $(($argn:ident $argt:ty))*] $ret:ty) $($body:tt)*)) => {
        fn $name(&mut $selph $(, $argn:$argt)*) -> $ret {
            $(lisp!($body));*
        }
    };
    ((defn $name:ident ([($selph:ident: Box<Self>) $(($argn:ident $argt:ty))*] $ret:ty) $($body:tt)*)) => {
        fn $name($selph: Box<Self> $(, $argn:$argt)*) -> $ret {
            $(lisp!($body));*
        }
    };
    ((defn $name:ident ([$(($argn:ident $argt:ty))*] $ret:ty) $($body:tt)*)) => {
        fn $name($($argn:$argt),*) -> $ret {
            $(lisp!($body));*
        }
    };

    ((defstruct $name:ident $(($typ:ty))*)) => {
        struct $name($($typ),*);
    };
    ((defstruct $name:ident $(($field:ident $typ:ty))*)) => {
        struct $name { $($field: $typ),* }
    };
    ((defstruct $name:ident <$($gen:ident)*>)) => {
        struct $name<$($gen),*>;
    };
    ((defstruct $name:ident <$($gen:ident)*> (where $(($wty:ident $($wtr:tt)*))*) $(($typ:ty))*)) => {
        struct $name<$($gen),*>($($typ),*) where $($wty: $($wtr)*),*;
    };
    ((defstruct $name:ident <$($gen:ident)*> (where $(($wty:ident $($wtr:tt)*))*) $(($field:ident $typ:ty))*)) => {
        struct $name<$($gen),*> where $($wty: $($wtr)*),* { $($field: $typ),* }
    };

    ((deftype $name:ident $typ:ty)) => { type $name = $typ; };
    ((deftype $name:ident <$($gen:ident)*> $typ:ty)) => { type $name<$($gen),*> = $typ; };

    ((defimpl ($trate:ty) (for $typ:ty) $($body:tt)*)) => {
        impl $trate for $typ {
            $(lisp!($body);)*
        }
    };
    ((defimpl <$($gen:ident)*> ($trate:ty) (for $typ:ty) (where $(($wty:ident $($wtr:tt)*))*) $($body:tt)*)) => {
        impl<$($gen),*> $trate for $typ where $($wty: $($wtr)*),* {
            $(lisp!($body);)*
        }
    };
    ((defimpl <$($gen:ident)*> ($trate:ty) (for $typ:ty) $($body:tt)*)) => {
        impl<$($gen),*> $trate for $typ where $($wty: $($wtr)*),* {
            $(lisp!($body);)*
        }
    };

    ((if $cond:tt $yes:tt $no:tt)) => {
        if lisp!($cond) { lisp!($yes) } else { lisp!($no) }
    };
    ((while $cond:tt $($body:tt)*)) => {
        while lisp!($cond) { $(lisp!($body));* }
    };
    // TODO for loops
    ((match $var:tt $(($cond:pat) $arm:tt)*)) => {
        match lisp!($var) {
            $($cond => lisp!($arm)),*
        }
    };
    ((do $($stmts:tt)*)) => {{
        $(lisp!($stmts));*
    }};

    // variables
    ((let [] $($body:tt)*)) => {{
        $(lisp!($body));*
    }};
    ((let [mut $var:ident $val:tt $($bindings:tt)*] $($body:tt)*)) => {{
        let mut $var = lisp!($val);
        lisp!((let [$($bindings)*] $($body)*))
    }};
    ((let [$var:ident $val:tt $($bindings:tt)*] $($body:tt)*)) => {{
        let $var = lisp!($val);
        lisp!((let [$($bindings)*] $($body)*))
    }};
    ((:= $var:ident $val:tt)) => {
        $var = lisp!($val);
    };

    // escape hatch
    ((rust $body:block)) => {
        { $body }
    };

    // list parsing
    (($($elem:tt)*)) => {
        lisp!(@list $($elem),*)
    };

    // parsers for unary and binary operators
    (@list -,    $arg:tt   ) => { lisp!(@unary  _neg,   $arg   ) };
    (@list !,    $arg:tt   ) => { lisp!(@unary  _not,   $arg   ) };
    (@list +,  $($arg:tt),*) => { lisp!(@binary _add, $($arg),*) };
    (@list &,  $arg:tt) => { &lisp!($arg) };
    (@list &,  $($arg:tt),*) => { lisp!(@binary _and, $($arg),*) };
    (@list |,  $($arg:tt),*) => { lisp!(@binary _or,  $($arg),*) };
    (@list ^,  $($arg:tt),*) => { lisp!(@binary _xor, $($arg),*) };
    (@list /,  $($arg:tt),*) => { lisp!(@binary _div, $($arg),*) };
    (@list *,  $arg:tt) => { *lisp!($arg) };
    (@list *,  $($arg:tt),*) => { lisp!(@binary _mul, $($arg),*) };
    (@list %,  $($arg:tt),*) => { lisp!(@binary _rem, $($arg),*) };
    (@list <<, $($arg:tt),*) => { lisp!(@binary _shl, $($arg),*) };
    (@list >>, $($arg:tt),*) => { lisp!(@binary _shr, $($arg),*) };
    (@list -,  $($arg:tt),*) => { lisp!(@binary _sub, $($arg),*) };
    (@list ==, $($arg:tt),*) => { lisp!(@binary _eq,  $($arg),*) };
    (@list !=, $($arg:tt),*) => { lisp!(@binary _ne,  $($arg),*) };
    (@list >,  $($arg:tt),*) => { lisp!(@binary _gt,  $($arg),*) };
    (@list <,  $($arg:tt),*) => { lisp!(@binary _lt,  $($arg),*) };
    (@list >=, $($arg:tt),*) => { lisp!(@binary _ge,  $($arg),*) };
    (@list <=, $($arg:tt),*) => { lisp!(@binary _le,  $($arg),*) };

    // generically turn unary/binary operators into function calls
    // binary operators can be used as n-ary operators through @reduce
    (@unary  $op:ident, $a:tt)        => { lisp!(@list $op, $a)     };
    (@binary $op:ident, $a:tt, $b:tt) => { lisp!(@list $op, $a, $b) };
    (@binary $op:ident, $a:tt, $b:tt, $($rest:tt),+) =>
                                               { lisp!(@reduce $op,
                                                       ($op $a $b),
                                                       $($rest),+) };

    // reduce implementation
    // TODO external entry point for @reduce
    (@reduce $op:ident, $acc:tt)                       => { lisp!($acc)          };
    (@reduce $op:ident, $acc:tt, $a:tt)                => { lisp!(@reduce $op,
                                                                  ($op $acc $a)) };
    (@reduce $op:ident, $acc:tt, $a:tt, $($rest:tt),+) => { lisp!(@reduce $op,
                                                                  ($op $acc $a),
                                                                  $($rest),+)    };
    // ns form helpers
    (@krate $(^$attr:meta)* $krate:ident) => {
        $(#[$attr:meta])*
        extern crate $krate;
    };
    (@krate $(^$attr:meta)* ($krate:ident $alias:ident)) => {
        $(#[$attr:meta])*
        extern crate $krate as $alias;
    };
    (@uze ($($head:ident)*)) => { use $($head)::*; };
    (@uze ($($head:ident)* { $($multiple:ident)* })) => {
        lisp!(@uze mult ($($head)*) { $($multiple)* });
    };
    (@uze mult $head:tt { $($multiple:ident)* }) => {
        $(lisp!(@uze mult out $head $multiple);)*
    };
    (@uze mult out ($($head:ident)*) $multiple:ident) => {
        use $($head)::*::$multiple;
    };

    // macro calls
    (@list $mac:ident, !) => {
        $mac!()
    };
    (@list $mac:ident, !, $($arg:tt),*) => {
        $mac!($(lisp!($arg)),*)
    };

    // struct constructors
    (@list (:: $($name:tt)*), .) => {
        $($name)*
    };
    (@list (:: $($name:tt)*), . $(, ($member:ident $val:tt))*) => {
        $($name)* { $($member: lisp!($val))* }
    };
    (@list $name:ident, . $(, ($member:ident $val:tt))*) => {
        $name { $($member: lisp!($val))* }
    };

    // function calls
    (@list (:: $name:path) $(, $arg:tt),*) => {
        $name($(lisp!($arg)),*)
    };
    (@list $name:expr $(, $arg:tt)*) => {
        lisp!($name)($(lisp!($arg)),*)
    };

    // method calls
    (@list ., $name:ident, $subj:tt $(, $arg:tt)*) => {
        lisp!($subj).$name($(lisp!($arg)),*)
    };

    // one expression
    ($e:expr) => ($e);
}
