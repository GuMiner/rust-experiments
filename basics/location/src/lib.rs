mod vec_ops;

// ==Other notes==
// Generics look very, very similar to C# with more ':' needed to fully define the type -- no runtime cost is present
// Traits are similar to interfaces, are allowed to implement traits for other external types (with restrictions)
//  -- Can have a deafult
//  -- Can be used to conditionally define methods for generic types if they match given type constraints.
// || {} defines closures / lambdas / whatever you want to call called functions. Basically they operate similar to the C++ equivalent.
// ' defines the lifetime parameter, so 'static can define string literals *&'static str* as living forever. 
// Box -- allocates on the heap. Rc -- reference counted type. Ref / RefMut (with RefCell for both) -- enfoces borrowing rules at runtime
//  All of these are other ways for dealing with smart pointers.
// Trait objects -- more like traditional List<IMyInterface> in C#. Much slower than normal Rust generics.

// Rust makes it much more difficult to have global state,
// which actually makes the programmability clearer (and threading a whole lot safer)
// because we pass back and forth the object we deal with.
pub struct CustomVector {
    vector: Vec<i32>,
}

pub fn create_vec() -> CustomVector {
    CustomVector { vector: Vec::new() } // vec![1,2,3] is another macro to simplify creating vectors
}

impl CustomVector {
    pub fn add_item(&mut self, item: i32) -> i32 {
        self.vector.push(item);
        println!("Added {}", item);
        return self.vector.len() as i32;
    }

    pub fn get_item(&self, idx: usize) -> &i32 {
        return &(self.vector)[idx];
    }

    pub fn remove_item(&mut self) -> Result<i32, &'static str> {
        match self.vector.len() {
            0 => Err("No more items to remove!"),
            _ => {
                let vector_len = self.vector.len() as i32;
                self.vector.remove((vector_len - 1) as usize);
                return Ok(self.vector.len() as i32);
            }
        }
    }
}

pub fn write_data() {
    let (x, y) = vec_ops::add(1, 2, 3, 4);
    println!("{} {} {}", 22, x, y);
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_test() {
        super::write_data();
    }
}
