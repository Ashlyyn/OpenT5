use num_derive::FromPrimitive;

#[derive(Copy, Clone, Default, FromPrimitive)]
#[repr(u8)]
enum Connstate {
    #[default]
    DISCONNECTED = 0,
    CINEMATIC = 1,
    UICINEMATIC = 2,
    LOGO = 3,
    CONNECTING = 4,
    CHALLENGING = 5,
    CONNECTED = 6,
    SENDINGSTATS = 7,
    LOADING = 8,
    PRIMED = 9,
    ACTIVE = 10,
}
