error_chain!{
    foreign_links {
        Io(::std::io::Error);
        Log(::log::SetLoggerError);
        Xcb(::xcb::GenericError);
        Xdg(::xdg::BaseDirectoriesError);
    }
}
