use super::*;
use crate::domain::terminal::{TerminalCell, TerminalSnapshot};
use alacritty_terminal::{
    event::WindowSize,
    event_loop::EventLoop,
    term::{Config, TermMode, cell::Flags, color::Colors, test::TermSize},
    tty,
    vte::ansi::{Color, NamedColor},
};
use anyhow::Result;
use std::path::PathBuf;

pub(super) fn start(
    root: PathBuf,
    command: Option<String>,
) -> Result<(Session, mpsc::UnboundedReceiver<Event>)> {
    let (tx, rx) = mpsc::unbounded_channel();
    let pty_exited = Arc::new(AtomicBool::new(false));
    let listener = Listener {
        events: tx,
        exited: pty_exited.clone(),
    };
    let size = TermSize::new(100, 12);
    let term = Arc::new(FairMutex::new(Term::new(
        Config::default(),
        &size,
        listener.clone(),
    )));
    let launch = shell::prepare(&root, command.as_deref())?;
    tracing::info!(shell = %launch.title, ?root, running_command = command.is_some(), "starting terminal shell");
    let pty = tty::new(
        &launch.options,
        WindowSize {
            num_lines: 12,
            num_cols: 100,
            cell_width: 8,
            cell_height: 18,
        },
        0,
    )?;
    if let Some(command) = &command {
        let text = format!("\x1b[34m{}\x1b[0m\r\n", command.replace('\n', "\r\n"));
        let mut parser: alacritty_terminal::vte::ansi::Processor = Default::default();
        parser.advance(&mut *term.lock(), text.as_bytes());
    }
    let process_id = process::id(&pty)?;
    let event_loop = EventLoop::new(term.clone(), listener, pty, true, false)?;
    let sender = event_loop.channel();
    let reader = Some(event_loop.spawn());
    static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let generation = GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Ok((
        Session {
            stopped: false,
            output_finalized: false,
            completion_written: false,
            cell_width: 8,
            cell_height: 18,
            pty_exited,
            process_id,
            reader,
            command,
            exit_code: None,
            generation,
            term,
            sender,
            title: launch.title.clone(),
            _launch: launch,
            exited: false,
        },
        rx,
    ))
}

pub(super) fn finish_output(session: &mut Session) {
    if !session.output_finalized
        || !session.exited
        || session.command.is_none()
        || session.completion_written
    {
        return;
    }
    session.completion_written = true;
    let message = session.exit_code.map_or_else(
        || "Process terminated".into(),
        |code| format!("Process finished with exit code {code}"),
    );
    let text = format!("\x1b[0m\r\n\r\n{message}\r\n");
    let mut parser: alacritty_terminal::vte::ansi::Processor = Default::default();
    parser.advance(&mut *session.term.lock(), text.as_bytes());
}

pub(super) fn snapshot(session: &Session) -> TerminalSnapshot {
    use alacritty_terminal::grid::Dimensions;
    let term = session.term.lock();
    let content = term.renderable_content();
    let columns = term.columns();
    let lines = term.screen_lines();
    let mut cells = vec![
        TerminalCell {
            text: " ".into(),
            foreground: 0x20242b,
            background: 0xffffff,
            ..Default::default()
        };
        columns * lines
    ];
    for indexed in content.display_iter {
        let row = indexed.point.line.0 + content.display_offset as i32;
        if row < 0 || row as usize >= lines {
            continue;
        }
        let cell = indexed.cell;
        let mut foreground = resolve_color(cell.fg, content.colors);
        let mut background = resolve_color(cell.bg, content.colors);
        if cell.flags.contains(Flags::INVERSE) {
            std::mem::swap(&mut foreground, &mut background);
        }
        let mut text = cell.c.to_string();
        if let Some(extra) = cell.zerowidth() {
            text.extend(extra);
        }
        if cell
            .flags
            .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER | Flags::HIDDEN)
        {
            text.clear();
        }
        cells[row as usize * columns + indexed.point.column.0] = TerminalCell {
            text,
            foreground,
            background,
            wide: cell.flags.contains(Flags::WIDE_CHAR),
            bold: cell.flags.contains(Flags::BOLD),
            italic: cell.flags.contains(Flags::ITALIC),
            underline: cell.flags.intersects(Flags::ALL_UNDERLINES),
        };
    }
    let cursor_row = content.cursor.point.line.0 + content.display_offset as i32;
    let cursor = if !session.exited
        && content.mode.contains(TermMode::SHOW_CURSOR)
        && cursor_row >= 0
        && (cursor_row as usize) < lines
    {
        Some((cursor_row as usize, content.cursor.point.column.0))
    } else {
        None
    };
    TerminalSnapshot {
        command: session.command.clone(),
        exit_code: session.exit_code,
        generation: session.generation,
        cells,
        columns,
        lines,
        history_size: term.grid().history_size(),
        scroll_offset: content.display_offset,
        cursor,
        title: session.title.clone(),
        exited: session.exited,
        application_cursor: content.mode.contains(TermMode::APP_CURSOR),
        bracketed_paste: content.mode.contains(TermMode::BRACKETED_PASTE),
    }
}

pub(super) fn resolve_color(value: Color, colors: &Colors) -> u32 {
    match value {
        Color::Spec(rgb) => ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | rgb.b as u32,
        Color::Named(name) => indexed_color(name as usize, colors),
        Color::Indexed(index) => indexed_color(index as usize, colors),
    }
}
pub(super) fn indexed_color(index: usize, colors: &Colors) -> u32 {
    if let Some(rgb) = colors[index] {
        return ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | rgb.b as u32;
    }
    const ANSI: [u32; 16] = [
        0x000000, 0xcc0000, 0x008000, 0xd6ab00, 0x0000ee, 0x800080, 0x008080, 0xc7c7c7, 0x808080,
        0xff0000, 0x00c000, 0xffc000, 0x5575ff, 0xc000c0, 0x00a0a0, 0xffffff,
    ];
    match index {
        0..=15 => ANSI[index],
        16..=231 => {
            let n = index - 16;
            let ramp = [0, 95, 135, 175, 215, 255];
            (ramp[n / 36] << 16) | (ramp[n / 6 % 6] << 8) | ramp[n % 6]
        }
        232..=255 => {
            let level = (8 + (index - 232) * 10) as u32;
            level * 0x010101
        }
        n if n == NamedColor::Background as usize => 0xffffff,
        _ => 0x000000,
    }
}
