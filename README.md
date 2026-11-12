## impl for Tuple

A macro that consumes a rust-like `impl for` block and generates implementations for a generic tuple

Special element and element type reference patterns would be converted to appropriate expressions/types

can be thought of as a `Tuple` pseudo-trait:
```rust
/// The pseudo trait for implementing traits on tuples
trait Tuple {
    /// The number of elements in the tuple
    const Length: usize;
    /// The type of the element in position `N`, translates directly to the appropriate type
    type Element<const N: usize>;
    /// Get the Nth element, translates to a field expression, invoked only using index operations, eg: &self[3] => &self.3
    fn get<const N: usize>() -> Self::Element<N>;
}
```
### `for` loops
It is possible to loop over tuple indices, this has some important uses

elements can be fetched from the tuple using indexing operators: `self[i]`, this also allows easily borrowing 
instead of owning when the self is a borrowed value, eg: `&self[i]`/`&mut self[i]`

Types can be evaluated using `Self::Element<{i}>`

This is a special loop type which is unrolled by the macro, as such break and continue are not supported (it may be possible to support them later using labels)

Within this loop, the index value is a *constant*, this is important since type/element accessors must use const indices

```rust
impl Print for Tuple {
    fn print(&self) {
        for i in 0..Self::Length {
            println!(&self[i])
        }
    }
}
```
would become:
```rust
impl<A, B, C> Print for (A, B, C) {
    fn print(&self) {
        println!(self.0);
        println!(self.1);
        println!(self.2);
    }
} 
```

### `where` clauses

Complex where clauses would allow it to become truly powerful:
- `Self::Length > 1`: limits impl blocks generated
- `Self::Element<0> = Type`: Hard codes first element as `Type`
- `Self::Element<1..15>: Debug`: Constrains all elements from 1 to 15 exclusive to `Debug`

### Examples
here are some examples showing an example block generated (in reality multiple would be generated)

```rust 
trait Clear {
    fn clear(&mut self);
}

impl_for_tuple! {
    impl Clear for Tuple 
    where Self::Element<*>: Clear
    {
        fn clear(&mut self) {
            for element in &mut self.elements {
                element.clear();
            }
        }
    }
}

// generated output
impl<A, B, C> Clear for (A, B, C) where A: Clear, B: Clear, C: Clear {
    fn clear(&mut self) {
        let element = &mut self.0;
        element.clear();
        let element = &mut self.1;
        element.clear();
        let element = &mut self.2;
        element.clear();
    }
} 
```

```rust 
trait Trait {
    type Ret;
    type Arg;
    type FixedType;
    const VALUE: u32;

    fn test(arg: Self::Arg) -> Self::Ret;

    fn test_with_self(&self) -> Result<(), ()>;
}

impl_for_tuple! {
    impl Trait for Tuple 
    where Self::Element<*>: Trait
    {
        type Ret = Self::Element<*>::Ret;
        type Arg = Self::Element<*>::Arg;
        const VALUE: u32 = {
            let mut sum = 0;
            for T in Self::Element<*> {
                sum += T::VALUE;
            }
            sum
        };
        
        fn test(arg: Self::Arg) -> Self::Ret {
            Self::map(arg, |T, arg| {
                T::test(arg)
            })
        }
        
        fn test_with_self(&self) -> Result<(), ()> {
            for element in &self.elements {
                element.test_with_self()?;
            }
            Ok(())
        }
    }
}

// generated output
impl<A, B, C> Trait for (A, B, C) where A: Trait, B: Trait, C: Trait {
    type Ret = (A::Ret, B::Ret, C::Ret);
    type Arg = (A::Arg, B::Arg, C::Arg);
    const VALUE: u32 = {
        let mut sum = 0;
        sum += A::VALUE;
        sum += B::VALUE;
        sum += C::VALUE;
        sum
    };
    
    fn test(arg: Self::Arg) -> Self::Ret {
        (
            {
                let arg = arg.0;
                A::test(arg)
            },
            {
                let arg = arg.1;
                B::test(arg)
            },
            {
                let arg = arg.2;
                C::test(arg)
            },
        )
    }
    
    fn test_with_self(&self) {
        let element = &self.0;
        element.test_with_self()?;
        let element = &self.1;
        element.test_with_self()?;
        let element = &self.2;
        element.test_with_self()?;
        Ok(())
    }
} 
```