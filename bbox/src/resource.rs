pub struct Resource {
    local_path: String,
    file_list: Vec<String>,
}

impl Resource {
    pub fn from() -> Resource {
        let mut resource = Resource {
            local_path: String::from("/Users/xxscloud/code/test/rust/bbox/src"),
            file_list: vec![],
        };
        resource.file_list.push(String::from("123"));
        resource
    }

    pub fn check_url(self, url: &str) -> bool {
        self.file_list.contains(&String::from(url))
    }

    pub fn read_file(self, url: &str) -> bool {
        let path = format!("/Users/xxscloud/code/test/rust/bbox/src/resource.rs");
        let file = std::fs::File::open("data.txt").unwrap();
        false
    }
}