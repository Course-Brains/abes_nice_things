#[cfg(test)]
mod tests {
    mod index_max {
        use abes_nice_things::PartialIterOrd;
        #[test]
        fn normal() {
            let list: Vec<usize> = vec![0, 2, 53, 4, 9];
            assert_eq!(
                list.iter().max().unwrap(),
                &list[list.clone().index_max()]
            );
        }
    }
}