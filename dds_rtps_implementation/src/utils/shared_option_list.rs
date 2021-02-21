use super::shared_option::SharedOption;

const LIST_SIZE: usize = 32;

pub struct SharedOptionList<T>([SharedOption<T>;LIST_SIZE]);

impl<T> Default for SharedOptionList<T> {
    fn default() -> Self {
        Default::default()
    }
}

