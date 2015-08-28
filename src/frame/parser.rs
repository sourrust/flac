use nom::be_u16;

use frame::Footer;

named!(footer <&[u8], Footer>, map!(be_u16, Footer));
