use super::*;

pub async fn start_info(m: Box<dyn BotMessage + Send + Sync>, json: String) {
    //println!("ACTION_PICKER: intent json is {}", json);
    let title_pass = general::title_pass_retriever(json);
    println!(
        "ACTION_PICKER: info title pass is {}, {}",
        title_pass.0, title_pass.1
    );
    if let Some(info) = golib::get_info(title_pass.0, title_pass.1) {
        (*m).send_message(MsgCount::SingleMsg(Msg::Text(info)));
    } else {
        responses::unsupported_notice(m);
    }
}
