use super::{access::*, Volatile};

#[test]
fn test_read() {
    let val = 42;
    assert_eq!(unsafe { Volatile::from_ptr(&val) }.read(), 42);
}

#[test]
fn test_write() {
    let mut val = 50;
    let mut volatile = unsafe { Volatile::from_mut_ptr(&mut val) };
    volatile.write(50);
    assert_eq!(val, 50);
}

#[test]
fn test_update() {
    let mut val = 42;
    let mut volatile = unsafe { Volatile::from_mut_ptr(&mut val) };
    volatile.update(|v| *v += 1);
    assert_eq!(val, 43);
}

#[test]
fn test_access() {
    let mut val: i64 = 42;

    // ReadWrite
    assert_eq!(
        unsafe { Volatile::new(&mut val, Access::read_write()) }.read(),
        42
    );
    unsafe { Volatile::new(&mut val, Access::read_write()) }.write(50);
    assert_eq!(val, 50);
    unsafe { Volatile::new(&mut val, Access::read_write()) }.update(|i| *i += 1);
    assert_eq!(val, 51);

    // ReadOnly and WriteOnly
    assert_eq!(
        unsafe { Volatile::new(&mut val, Access::read_only()) }.read(),
        51
    );
    unsafe { Volatile::new(&mut val, Access::write_only()) }.write(12);
    assert_eq!(val, 12);

    // Custom: safe read + safe write
    {
        let access = Access {
            read: SafeAccess,
            write: SafeAccess,
        };
        let mut volatile = unsafe { Volatile::new(&mut val, access) };
        let random: i32 = rand::random();
        volatile.write(i64::from(random));
        assert_eq!(volatile.read(), i64::from(random));
        let random2: i32 = rand::random();
        volatile.update(|i| *i += i64::from(random2));
        assert_eq!(volatile.read(), i64::from(random) + i64::from(random2));
    }

    // Custom: safe read + unsafe write
    {
        let access = Access {
            read: SafeAccess,
            write: UnsafeAccess,
        };
        let mut volatile = unsafe { Volatile::new(&mut val, access) };
        let random: i32 = rand::random();
        unsafe { volatile.write_unsafe(i64::from(random)) };
        assert_eq!(volatile.read(), i64::from(random));
        let random2: i32 = rand::random();
        unsafe { volatile.update_unsafe(|i| *i += i64::from(random2)) };
        assert_eq!(volatile.read(), i64::from(random) + i64::from(random2));
    }

    // Custom: safe read + no write
    {
        let access = Access {
            read: SafeAccess,
            write: NoAccess,
        };
        let random = rand::random();
        val = random;
        let mut volatile = unsafe { Volatile::new(&mut val, access) };
        assert_eq!(volatile.read(), i64::from(random));
    }

    // Custom: unsafe read + safe write
    {
        let access = Access {
            read: UnsafeAccess,
            write: SafeAccess,
        };
        let mut volatile = unsafe { Volatile::new(&mut val, access) };
        let random: i32 = rand::random();
        volatile.write(i64::from(random));
        assert_eq!(unsafe { volatile.read_unsafe() }, i64::from(random));
        let random2: i32 = rand::random();
        unsafe { volatile.update_unsafe(|i| *i += i64::from(random2)) };
        assert_eq!(
            unsafe { volatile.read_unsafe() },
            i64::from(random) + i64::from(random2)
        );
    }

    // Todo: Custom: unsafe read + unsafe write
    // Todo: Custom: unsafe read + no write
    // Todo: Custom: no read + safe write
    // Todo: Custom: no read + unsafe write
    // Todo: Custom: no read + no write

    // Todo: is there a way to check that a compile error occurs when trying to use
    // unavailable methods (e.g. `write` when write permission is `UnsafeAccess`)?
}

#[test]
fn test_struct() {
    #[derive(Debug, PartialEq)]
    struct S {
        field_1: u32,
        field_2: bool,
    }

    let mut val = S {
        field_1: 60,
        field_2: true,
    };
    let mut volatile = unsafe { Volatile::from_mut_ptr(&mut val) };
    unsafe { volatile.map_mut(|s| core::ptr::addr_of_mut!((*s).field_1)) }.update(|v| *v += 1);
    let mut field_2 = unsafe { volatile.map_mut(|s| core::ptr::addr_of_mut!((*s).field_2)) };
    assert!(field_2.read());
    field_2.write(false);
    assert_eq!(
        val,
        S {
            field_1: 61,
            field_2: false
        }
    );
}

#[test]
fn test_struct_macro() {
    #[derive(Debug, PartialEq)]
    struct S {
        field_1: u32,
        field_2: bool,
    }

    let mut val = S {
        field_1: 60,
        field_2: true,
    };
    let mut volatile = unsafe { Volatile::from_mut_ptr(&mut val) };
    let mut field_1 = map_field_mut!(volatile.field_1);
    field_1.update(|v| *v += 1);
    let mut field_2 = map_field_mut!(volatile.field_2);
    assert!(field_2.read());
    field_2.write(false);
    assert_eq!(
        val,
        S {
            field_1: 61,
            field_2: false
        }
    );
}

#[cfg(feature = "unstable")]
#[test]
fn test_slice() {
    let mut val: &mut [u32] = &mut [1, 2, 3];
    let mut volatile = unsafe { Volatile::from_mut_ptr(val) };
    volatile.index_mut(0).update(|v| *v += 1);
    assert_eq!(val, [2, 2, 3]);
}

#[cfg(feature = "unstable")]
#[test]
fn test_chunks() {
    let mut val: &mut [u32] = &mut [1, 2, 3, 4, 5, 6];
    let mut volatile = unsafe { Volatile::from_mut_ptr(val) };
    let mut chunks = volatile.as_chunks_mut().0;
    chunks.index_mut(1).write([10, 11, 12]);
    assert_eq!(chunks.index(0).read(), [1, 2, 3]);
    assert_eq!(chunks.index(1).read(), [10, 11, 12]);
}
