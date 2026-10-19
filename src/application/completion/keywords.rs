pub(super) fn words(language: &str) -> &'static str {
    match language {
        "rust" => {
            "as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while bool char str String Vec Option Result Some None Ok Err u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 pub(crate) pub(super)"
        }
        "python" => {
            "and as assert async await break class continue def del elif else except False finally for from global if import in is lambda None nonlocal not or pass raise return True try while with yield self print len range str int float list dict set tuple bool super"
        }
        "javascript" | "typescript" | "tsx" => {
            "async await break case catch class const continue debugger default delete do else export extends false finally for from function if import in instanceof let new null of return static super switch this throw true try typeof undefined var void while with yield console document window Promise Array Object String Number Boolean Map Set interface type enum implements private protected public readonly abstract declare namespace keyof never unknown any string number boolean"
        }
        "java" => {
            "abstract assert boolean break byte case catch char class const continue default do double else enum extends final finally float for if implements import instanceof int interface long native new null package private protected public return short static strictfp super switch synchronized this throw throws transient true false try void volatile while String System Integer Override"
        }
        "go" => {
            "break case chan const continue default defer else fallthrough for func go goto if import interface map package range return select struct switch type var bool byte complex64 complex128 error float32 float64 int int8 int16 int32 int64 rune string uint uint8 uint16 uint32 uint64 uintptr true false nil append cap close copy delete len make new panic print println recover"
        }
        "c" | "cpp" => {
            "auto break case char const continue default do double else enum extern float for goto if int long register return short signed sizeof static struct switch typedef union unsigned void volatile while bool true false NULL nullptr class namespace public private protected template typename virtual override final constexpr consteval using new delete this throw try catch include define ifdef endif std string vector size_t"
        }
        "cmake" => {
            "cmake_minimum_required project add_executable add_library target_link_libraries target_include_directories target_compile_options target_compile_definitions set option if elseif else endif foreach endforeach function endfunction macro endmacro find_package include message install ON OFF TRUE FALSE"
        }
        "html" => {
            "html head body title meta link script style div span main header footer nav section article aside h1 h2 h3 p a img button input form label select option textarea ul ol li table tr td th class id href src alt type name value disabled placeholder"
        }
        "css" => {
            "display position relative absolute fixed sticky color background border padding margin width height min-width max-width min-height max-height flex grid gap align-items justify-content font-family font-size font-weight line-height overflow auto none block inline-block solid transparent inherit important"
        }
        "php" => {
            "abstract array as break callable case catch class clone const continue declare default die do echo else elseif empty enddeclare endfor endforeach endif endswitch endwhile enum eval exit extends final finally fn for foreach function global goto if implements include include_once instanceof insteadof interface isset list match namespace new null parent print private protected public readonly require require_once return self static switch throw trait true false try unset use var while yield string int float bool mixed void never"
        }
        "json" => "true false null",
        "toml" | "yaml" => "true false",
        _ => "",
    }
}
