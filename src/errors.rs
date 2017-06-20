error_chain!{
    foreign_links {
        Fern(::fern::InitError);
        Xcb(::xcb::GenericError);
        Xdg(::xdg::BaseDirectoriesError);
    }
}
