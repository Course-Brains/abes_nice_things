use std::convert::Infallible;
pub struct Input<'a, E = Infallible> {
    msg: Option<&'a str>,
    cond: Option<Box<dyn Fn(&String) -> Result<bool, E>>>
}
impl<'a, E> Input<'a, E> {
    pub fn get(&self) -> Result<String, E> {
        loop {
            if let Some(msg) = self.msg {
                println!("{msg}")
            }

            let mut string: String = String::new();
            std::io::stdin().read_line(&mut string).unwrap();
            if let Some('\n'|'\r') = string.chars().next_back() {
                string.pop();
            }

            return Ok(match &self.cond {
                Some(cond) => {
                    if cond(&string)? {
                        string
                    }
                    else {
                        continue
                    }
                }
                None => string
            })
        }
    }
    pub fn new() -> Self {
        Self { ..Default::default() }
    }
    pub fn yn() -> Self {
        Self {
            cond: Some(Box::new(|string: &String| {
                match string.as_str() {
                    "y" => return Ok(true),
                    "n" => return Ok(true),
                    _ => return Ok(false)
                }
            })),
            ..Default::default()
        }
    }
    pub fn msg(&mut self, msg: &'a str) -> &mut Self {
        self.msg = Some(msg);
        self
    }
    pub fn clear_msg(&mut self) -> &mut Self {
        self.msg = None;
        self
    }
    pub fn cond<F: Fn(&String) -> Result<bool, E> + 'static>(&mut self, cond: F) -> &mut Self {
        self.cond = Some(Box::new(cond));
        self
    }
    pub fn clear_cond(&mut self) -> &mut Self {
        self.cond = None;
        self
    }
}
impl<'a, E> Default for Input<'a, E> {
    fn default() -> Self {
        Self {
            msg: None,
            cond: None,
        }
    }
}
