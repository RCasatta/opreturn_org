use crate::now;
use maud::{html, Markup, DOCTYPE};

/// Pages headers.
fn header() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            script src="https://cdn.jsdelivr.net/npm/chart.js" { }

            title { "OpReturn" }
        }
    }
}

/*
<script src="https://codepen.io/anon/pen/aWapBE.js"></script>
<script type="text/javascript">

</script>
*/

/// A static footer.
fn footer() -> Markup {
    html! {
        footer {
            p { a href="/" { "Home" } " | " a href="/about" { "About" } " | " a href="/contact" { "Contact" }  }
            p { "Page created " (now()) }
        }

    }
}

/// The final Markup, including `header` and `footer`.
///
/// Additionally takes a `greeting_box` that's `Markup`, not `&str`.
pub fn page(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang = "en" {
            (header())
            body {
                h1 { a href="/" { "OpReturn" } }
                (content)
                (footer())
            }
        }
    }
}

pub fn create_contact() -> Markup {
    let content = html! {
        h2 { "Contact" }
        form action="https://formspree.io/f/xnqlrbey" method="POST" {
            label {
                p { "Your email:"}
                input type="email" name="_replyto" { }
            }
            br {}
            label {
                p { "Your message:"}
                textarea name="message" rows="4" cols="50" { }
            }
            input type="hidden" name="_tags" value="opreturn.org" { }
            br {}
            button type="submit" { "Send" }
            br {}
        }
    };

    page(content)
}

/*
#[cfg(test)]
mod test {
    use crate::message::test::{get_another_message, get_message};
    use crate::templates::{create_detail_page, create_index_page, create_list_page, page};
    use crate::MessagesByCat;
    use maud::html;
    use std::collections::BTreeSet;
    use whatlang::detect_lang;

    #[test]
    fn test_page() {
        let content = html! { p { "Hello" } };
        let page = page(content).into_string();
        assert_eq!("", to_data_url(&page, "text/html"));
    }

    #[test]
    fn test_escape() {
        let a = html! { p { "<>" } };
        assert_eq!(a.into_string(), "<p>&lt;&gt;</p>");
    }

    #[test]
    fn test_page_detail() {
        let msg = get_message();
        let page = create_detail_page(&msg);
        assert_eq!("", page);
        assert_eq!("", to_data_url(&page, "text/html"));
    }

    #[test]
    fn test_page_index() {
        let mut map = MessagesByCat::new();
        map.insert("2019".to_string(), BTreeSet::new());
        map.insert("2020".to_string(), BTreeSet::new());

        let page = create_index_page(&map, true);
        assert_eq!("", to_data_url(&page, "text/html"));
    }

    #[test]
    fn test_page_year() {
        let mut set = BTreeSet::new();
        set.insert(get_message());
        set.insert(get_another_message());
        let page = create_list_page("2020", set);
        assert_eq!("", to_data_url(&page, "text/html"));
    }

    #[test]
    fn test_lang() {
        assert_eq!(get_message().lang(), Some("en"));
        assert_eq!(get_another_message().lang(), Some("it"));
        let text = "洪沛东谢家霖自习课经常说话，纪律委员金涵笑大怒";
        println!("{:?}", detect_lang(text));
    }


}


 */
