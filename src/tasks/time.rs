#[cfg(not(test))]
pub fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

#[cfg(test)]
pub mod mock {
    use std::cell::RefCell;

    thread_local! {
        static MOCK_TIME: RefCell<Option<chrono::NaiveDate>> = RefCell::new(None);
    }

    pub fn today() -> chrono::NaiveDate {
        MOCK_TIME.with(|cell| {
            cell.borrow()
                .as_ref()
                .cloned()
                .expect("Fake date must be set for tests")
        })
    }

    pub fn set(time: chrono::NaiveDate) {
        MOCK_TIME.with(|cell| *cell.borrow_mut() = Some(time));
    }
}

#[cfg(test)]
pub use mock::today;
