/// This is a type to make getting input from the terminal easier. It even gets rid of the newline
/// for you! The basic usage is:
/// ```no_run
/// # use abes_nice_things::Input;
/// # fn main() {
/// let input = <Input>::new().get();
/// # }
/// ```
/// Although, if you are going to do that then I would recommend just using [input]
/// which makes getting a [String] of input from the terminal easier than using this.
///
/// What this is good at is conditionally giving back input from the user if it meets your
/// conditions and printing a prompt every time you try to get input.
///
/// # The message
/// The message is a bit of text which is printed just before this gets input from the user in order
/// to instruct them on what to type. The message is printed with a newline so the user will be
/// typing on the line below the message. It is specified as such:
/// ```no_run
/// # use abes_nice_things::Input;
/// # fn main() {
/// let input = <Input>::new().msg("Gimme a String!").get();
/// # }
/// ```
/// # The mapper
/// The mapper has two roles. It determines whether or not it needs to get another input from the
/// user. It also does any needed conversion from the inputted [String] to whatever output you
/// need.
///
/// As an example, this is the mapper set by the method [yn](Input::yn)
/// ```no_run
/// # use abes_nice_things::Input;
/// # fn main() {
/// let input = <Input>::new().mapper(|string| {
///     match string.as_str() {
///         "y" => Some(true),
///         "n" => Some(false),
///         _ => None
///     }
/// }).get();
/// # }
/// ```
/// As shown, the mapper closure takes in a [String] and returns an [Option].
/// If it returns Some, then it will stop getting input and pass the value in the Some out to the
/// caller. However, if it returns None then it will try again to get input from the user. An
/// important detail is that every time it tries to get input from the terminal, it prints out the
/// message, if there is one.
pub struct Input<T = String> {
    msg: Option<String>,
    mapper: Box<dyn Fn(String) -> Option<T>>,
}
impl<T> Input<T> {
    /// This actually gets input from the user with the settings you have chosen. If you are trying
    /// to figure out how to actually trigger it to get input, this is how.
    ///
    /// It isn't complicated, you set your settings with the other methods, then call this one and
    /// it'll do the stuff.
    pub fn get(&self) -> T {
        loop {
            if let Some(msg) = &self.msg {
                println!("{msg}")
            }

            let mut string: String = String::new();
            std::io::stdin().read_line(&mut string).unwrap();
            // removing newline
            string.pop();
            // removing the other char of newline on windows
            // because on windows it is \n\r
            #[cfg(target_os = "windows")]
            string.pop();

            match (self.mapper)(string) {
                Some(out) => break out,
                None => continue,
            }
        }
    }
    /// This creates an instance of [Input] with no message and no mapper. If you do not understand
    /// what those mean, read the docs for the type itself.
    ///
    /// There is one special part about this, because of complicated generic things that I will not
    /// be explaining in this, the easiest way to run this is with < and > around [Input] like so
    /// ```
    /// # use abes_nice_things::Input;
    /// # fn main() {
    /// let input = <Input>::new();
    /// # }
    /// ```
    /// If you want to know more about why that is a good idea, try running it without it and snoop
    /// around.
    pub fn new() -> Input<String> {
        Input {
            ..Default::default()
        }
    }
    /// This creates a new [Input] instance except that it will repeatedly try to get either 'y' or
    /// 'n' from the user and it will give back [true] and [false] for them respectively.
    ///
    /// The way you use it is the same as with [new](Input::new) so if you want to know how, go
    /// read that.
    pub fn yn() -> Input<bool> {
        <Input>::new().mapper(|input| match input.as_str() {
            "y" => Some(true),
            "n" => Some(false),
            _ => None,
        })
    }
    /// This sets the message used by the input. It can take anything that implements [ToString]
    ///
    /// The message is printed directly before every attempt to get input from the user. As a
    /// result, if the mapper causes it to reattempt getting input, it will reprint the message.
    ///
    /// Fun fact: [&str] implements [ToString]
    pub fn msg<S: ToString>(mut self, msg: S) -> Self {
        self.msg = Some(msg.to_string());
        self
    }
    /// This clears the message. If there is no message, then nothing is printed before getting
    /// input from the user, even an empty newline.
    pub fn clear_msg(mut self) -> Self {
        self.msg = None;
        self
    }
    /// This defines the mapper. The mapper has two roles.
    ///
    /// It needs to determine if the inputted [String] is valid, and if it isn't, then get input
    /// from the user again.
    ///
    /// It also needs to convert from the inputted [String] to whatever type you are trying to get
    /// from the terminal.
    ///
    /// As an example, this is the mapper used by [yn](Input::yn)
    /// ```
    /// # use abes_nice_things::Input;
    /// # fn main() {
    /// <Input>::new().mapper(|string| {
    ///     match string.as_str() {
    ///         "y" => Some(true),
    ///         "n" => Some(false),
    ///         _ => None
    ///     }
    /// });
    /// # }
    /// ```
    /// Explanation: If the user inputs "y", then it stops getting input and returns [true], if the
    /// user inputs "n" then it stops getting input and returns [false], and if anything else is
    /// inputted then it will try again and get more input then run the mapper on that.
    ///
    /// Because the mapper can return anything so long as you get it from the inputted string, you
    /// can even return a [Result] to handle errors! Just remember that if you make the mapper
    /// return a value instead of [None] then it will stop trying to get input from the user.
    pub fn mapper<O>(self, mapper: impl Fn(String) -> Option<O> + 'static) -> Input<O> {
        Input {
            msg: self.msg,
            mapper: Box::new(mapper),
        }
    }
    /// This removes the mapper. This makes it so that the [Input] will just return whatever the
    /// user inputs instead of doing any checks or conversions.
    pub fn clear_mapper(self) -> Input<String> {
        Input {
            msg: self.msg,
            mapper: Box::new(|string| Some(string)),
        }
    }
}
impl Default for Input<String> {
    fn default() -> Self {
        Self {
            msg: None,
            mapper: Box::new(|string| Some(string)),
        }
    }
}
/// This gets input from the user through the terminal.
/// It will wait for them to press enter(new line),
/// then will return whatever they typed as a [String],
/// without the trailing new line(\n or \r).
/// For instance, if you typed this in the terminal:
/// ```terminal
/// Hello! I am an example!
/// ```
/// Then, this would return the [String]:
/// "Hello! I am an example!"
///
/// This is a convenience function which is equivalent
/// to using [Input] in the following way:
/// ```no_run
/// # use abes_nice_things::Input;
/// # fn main() {
/// <Input>::new().get()
/// # ;
/// # }
/// ```
/// For more information, see the type level
/// [documentation](Input)
pub fn input() -> String {
    <Input>::new().get()
}
