use crate::assert_semantics;

// Reads

assert_semantics! {
    ok_reference_read_global, "let a/*#A*/ = 1; let b = a/*READ A*/ + 1;",

    ok_reference_read_inner_scope, r#"function f(a/*#A1*/) {
    let b = a/*READ A1*/ + 1;
    console.log(b);
    if (true) {
        let a/*#A2*/ = 2;
        let b = a/*READ A2*/ + 1;
        console.log(b);
    }
    let c = a/*READ A1*/ + 1;
    console.log(b);
}
f(1);"#,
}

// Read Hoisting

assert_semantics! {
    ok_hoisting_read_inside_function, "function f() {
    a = 2;
    let b = a/*READ A*/ + 1;
    console.log(a, b);
    
    var a/*#A*/;
}
f();",
    ok_hoisting_read_var_inside_if, r#"function f() {
    a = 1;
    let b = a/*READ A*/ + 1;
    console.log(a, b);
    if (true) {  
        var a/*#A*/;      
    }
}
f();"#,
    ok_hoisting_read_redeclaration_before_use, r#"var a/*#A1*/ = 1;
function f() {
    var a/*#A2*/ = 10;
    console.log(a/*READ A2*/);
}
f();"#,

ok_hoisting_read_redeclaration_after_use, r#"var a/*#A1*/ = 1;
function f() {
    console.log(a/*READ A2*/);
    var a/*#A2*/ = 10;
}
f();"#,

        ok_hoisting_read_for_of, r#"function f() {
    for (var a/*#A*/ of [1,2,3]) {
        console.log(a/*READ A*/)
    }
    console.log(a/*READ A*/);
}
f()"#,

    ok_hoisting_read_for_in, r#"function f() {
    for (var a/*#A*/ in [1,2,3]) {
        console.log(a/*READ A*/)
    }
    console.log(a/*READ A*/);
}
f()"#,

    ok_hoisting_read_let_after_reference_same_scope, r#"var a = 1;
function f() {
    console.log(a/*READ A*/);
    let a/*#A*/ = 2;
}
f()"#,

ok_hoisting_read_let_after_reference_different_scope, r#"var a/*#A*/ = 1;
function f() {
    console.log(a/*READ A*/);
    if (true) {
        let a = 2;
    }
}
f()"#,
}

// Write

assert_semantics! {
    ok_reference_write_global, "let a/*#A*/; a/*WRITE A*/ = 1;",
    ok_reference_write_inner_scope, r#"function f(a/*#A1*/) {
    a/*WRITE A1*/ = 1;
    console.log(a);
    if (true) {
        let a/*#A2*/;
        a/*WRITE A2*/ = 2;
        console.log(a);
    }
    a/*WRITE A1*/ = 3;
    console.log(3);
}
f(1);"#,
}

// Write Hoisting

assert_semantics! {
    ok_hoisting_write_inside_function, "function f() {
    a/*WRITE A*/ = 2;
    console.log(a);
    var a/*#A*/;
}
f();",
}

assert_semantics! {
    ok_unmatched_reference, r#"a/*?*/"#,
}
