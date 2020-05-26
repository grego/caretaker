use custom_error::custom_error;

custom_error! { pub Error
    Notify{source: notify::Error} = "notify error",
    Pattern{source: glob::PatternError} = "invalid glob",
    IO{source: std::io::Error} = "input / output error",
    Receive{source: crossbeam_channel::RecvError} = "channel receive error",
}
