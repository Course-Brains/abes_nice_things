use std::convert::Infallible;
/// This is a type which is similar to [OpenOptions](std::fs::OpenOptions)
/// and is used to get input from the terminal.
/// The simplest way to use this is to just get input with no
/// message or condition:
/// ```no_run
/// # use abes_nice_things::Input;
/// # fn main() {
/// let input: String = Input::new().get().unwrap();
/// # }
/// ```
/// As you may have noticed, we need to [unwrap](Result::unwrap)
/// the what is returned after we get input from the terminal.
/// In order to explain that I have to explain the various options
/// for this type.
/// # msg
/// You can define a message to be displayed every time
/// it attempts to get information from the terminal.
/// This message will be [printed](println) above where
/// they actually input data:
/// ```terminal
/// [Your message]
/// [where they would be typing]
/// ```
/// This message does have to be constant, but you can
/// define it after creating an [Input] instance
/// by using the [msg](Input::msg) method.
/// ```
/// # use abes_nice_things::Input;
/// # fn main() {
/// Input::new().msg("Whatever you want");
/// # }
/// ```
/// Notably, if no message is provided,
/// running [get](Input::get) will not
/// cause a newline to be [printed](println),
/// meaning that it will not cause a gap such as
/// ```terminal
/// 
/// [where you type]
/// ```
/// to occur.
/// # cond
/// While being able to define a message is nice,
/// sometimes you need to restrict the possible
/// responses or sometimes getting the input
/// itself can fai to meet your conditions.
/// To allow this to happen, you can give a 
/// closure that will be run with the [String]
/// given by the user as an input.
/// This closure must return either [Ok] containing
/// a bool indicating whether or not it is done
/// getting input. Specifically, if [true] is returned
/// by the closure, [get](Input::get) will return
/// [Ok] containing the string passed to the closure.
/// However, if [false] is returned by the closure,
/// it will re-attempt to get input from the terminal
/// and will re-send the message.
/// For instance, if only returning y or n was valid,
/// then this could be what would happen with the
/// message of "Give me y or n"
/// ```terminal
/// Give me y or n
/// I don't want to
/// Give me y or n
/// Y
/// Give me y or n
/// y
/// ```
/// As is shown, the the default condition given by
/// [yn](Input::yn) is case sensitive, which caused
/// it to only stop after being given exactly 'y' or
/// 'n' not 'Y' or 'N'. However, if you wanted to fix
/// this, you could make your own condition:
/// ```
/// # use abes_nice_things::Input;
/// # fn main() {
/// Input::new().cond(& |string| {
///     match string.as_str().to_lowercase() {
///         "y"|"n" => return Ok(true),
///         _ => return Ok(false)
///     }
/// });
/// # }
/// ```
/// You can set either of them while chaining method
/// calls together because they all return &mut [Input]
/// ```no_run
/// # use abes_nice_things::Input;
/// # fn main() {
/// let input: String = Input::new()
///     .msg("some message")
///     .cond(&|string| {
///         // Some condition
///         # return Ok(true)
///     })
///     .get().unwrap();
/// # }
/// ```
/// If needed, the condition can return an error,
/// which will stop all attempts to get input and
/// will return that error. However, because most
/// of the time, that is not needed, the default
/// error type is [Infallible].
pub struct Input<'a, E = Infallible> {
    msg: Option<&'a str>,
    cond: Option<&'a dyn Fn(&String) -> Result<bool, E>>
}
impl<'a, E> Input<'a, E> {
    /// This gets input from the terminal using the
    /// given settings. It does return a [Result],
    /// so in order to access the actual input, you
    /// will need to handle it, most of the time by
    /// [unwrap](Result::unwrap)ing it.
    /// ```no_run
    /// # use abes_nice_things::Input;
    /// # fn main() {
    /// let input: String = Input::new().get().unwrap();
    /// # }
    /// ```
    /// Notably, the [String] will be returned *without*
    /// the trailing new line.
    ///
    /// For more information see the type level
    /// [documentation](Input)
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
    /// This creates an instance of [Input]
    /// with the default settings of no
    /// message and no condition.
    ///
    /// For more information see the type level
    /// [documentation](Input)
    pub fn new() -> Self {
        Self { ..Default::default() }
    }
    /// This creates an instance of [Input]
    /// with settings of no message, and
    /// a condition that will only allow input
    /// which is either "y" or "n". Notably
    /// this is case sensitive.
    ///
    /// For more information, see the type level
    /// [documentation](Input)
    pub fn yn() -> Self {
        Self {
            cond: Some(&|string: &String| {
                match string.as_str() {
                    "y"|"n" => return Ok(true),
                    _ => return Ok(false)
                }
            }),
            ..Default::default()
        }
    }
    /// This defines the printed message.
    /// This message will be printed on the line
    /// above where the input is given
    /// ```terminal
    /// [message is here]
    /// [input is here]
    /// ```
    /// In order to set the message, you need
    /// to give this method a &[str] with the message
    /// you want printed. For example:
    /// ```
    /// # use abes_nice_things::Input;
    /// # fn main() {
    /// Input::new().msg("Something idk").get().unwrap();
    /// # }
    /// ```
    /// will cause the following in the terminal
    /// ```terminal
    /// Something idk
    /// [area to be typed in]
    /// ```
    /// For more information, see the type level
    /// [documentation](Input)
    pub fn msg(&mut self, msg: &'a str) -> &mut Self {
        self.msg = Some(msg);
        self
    }
    /// This will clear any message that has been
    /// given to [Input] and will revert it to
    /// the state of not having a message.
    /// Meaning that it will not print an empty
    /// line.
    ///
    /// For more information, see the type level
    /// [documentation](Input)
    pub fn clear_msg(&mut self) -> &mut Self {
        self.msg = None;
        self
    }
    /// This will set the condition used by [Input]
    /// The condition must take in the string given
    /// to the terminal(without the trailing newline
    /// of course). It must return a [Result] with
    /// either [Ok] containing a [bool] representing
    /// whether or not you are done getting input.
    /// Specifically, if [true] is given, it will stop
    /// collecting input and will return the [String]
    /// which resulted in [true] being returned by the
    /// closure. While [false] will cause it to repeat
    /// the input getting sequence again until [true]
    /// is returned(each run being done with the results
    /// of getting input from the terminal each time,
    /// meaning that each subsequent run requires another
    /// input to the terminal from the user).
    /// For example, the condition used
    /// by [yn](Input::yn) to enforce only y or n being
    /// returned could be done as shown
    /// ```
    /// # use abes_nice_things::Input;
    /// # fn main() {
    /// let input: String = Input::new()
    ///     .cond(&|string| {
    ///         match string {
    ///             "y"|"n" => return Ok(true),
    ///             _ => return Ok(false)
    ///         }
    ///     })
    ///     .get().unwrap()
    /// # }
    /// ```
    /// Notably, [Ok] is returned no matter what in this
    /// example, that is because there is no reason to
    /// fully stop getting input no matter what input
    /// is given from the user. If that is not the case
    /// when you use this, you can return [Err] and
    /// the error you want to be returned from [get](Input::get).
    /// If you are curious, the default error type is [Infallible]
    ///
    /// For more information, see the type level [documentation](Input)
    pub fn cond<F: Fn(&String) -> Result<bool, E> + 'static>(&mut self, cond: &'a F) -> &mut Self {
        self.cond = Some(cond);
        self
    }
    /// This will clear any condition set by
    /// [cond](Input::cond) and will return it
    /// to the state of having no checks,
    /// meaning it will return whatever is
    /// given by the user.
    ///
    /// For more information, see the type level
    /// [documentation](Input)
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
