
use std::io::{self, Write};
use std::sync::Arc;

use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    widgets::Paragraph,
    Terminal, TerminalOptions, Viewport,
};
use russh::{
    server::{self, Handler, Server, Session},
    Channel, ChannelId, Pty,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

type SshTerminal = Terminal<CrosstermBackend<SshWriter>>;

// Envia a saída visual do Ratatui pelo SSH.
struct SshWriter {
    sender: UnboundedSender<Vec<u8>>,
    buffer: Vec<u8>,
}

impl SshWriter {
    fn new(handle: server::Handle, channel: ChannelId) -> Self {
        let (sender, mut receiver) = unbounded_channel::<Vec<u8>>();

        tokio::spawn(async move {
            while let Some(data) = receiver.recv().await {
                if handle.data(channel, data).await.is_err() {
                    break;
                }
            }
        });

        Self {
            sender,
            buffer: Vec::new(),
        }
    }
}

impl Write for SshWriter {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            self.sender
                .send(std::mem::take(&mut self.buffer))
                .map_err(io::Error::other)?;
        }
        Ok(())
    }
}

// Interface da landing page.
fn render(terminal: &mut SshTerminal) -> io::Result<()> {
    terminal.draw(|frame| {
        let area = frame.area();

        let center = Rect::new(
            area.x,
            area.y + area.height / 2,
            area.width,
            1,
        );

        let text = Paragraph::new("tuiminal")
            .alignment(Alignment::Center);

        frame.render_widget(text, center);
    })?;

    Ok(())
}

// Cada visitante recebe sua própria sessão.
#[derive(Default)]
struct Client {
    terminal: Option<SshTerminal>,
}

impl Handler for Client {
    type Error = anyhow::Error;

    // Permite visitar a LP sem senha.
    async fn auth_none(
        &mut self,
        _user: &str,
    ) -> Result<server::Auth, Self::Error> {
        Ok(server::Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<server::Msg>,
        reply: server::ChannelOpenHandle,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let writer = SshWriter::new(session.handle(), channel.id());
        let backend = CrosstermBackend::new(writer);

        self.terminal = Some(Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Fixed(Rect::default()),
            },
        )?);

        reply.accept().await;
        Ok(())
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _term: &str,
        cols: u32,
        rows: u32,
        _width: u32,
        _height: u32,
        _modes: &[(Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(terminal) = &mut self.terminal {
            terminal.resize(Rect::new(
                0,
                0,
                cols.min(u16::MAX as u32) as u16,
                rows.min(u16::MAX as u32) as u16,
            ))?;
            session.channel_success(channel)?;
        } else {
            session.channel_failure(channel)?;
        }
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(terminal) = &mut self.terminal {
            session.channel_success(channel)?;
            render(terminal)?;
        } else {
            session.channel_failure(channel)?;
        }
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        _channel: ChannelId,
        cols: u32,
        rows: u32,
        _width: u32,
        _height: u32,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(terminal) = &mut self.terminal {
            terminal.resize(Rect::new(
                0,
                0,
                cols.min(u16::MAX as u32) as u16,
                rows.min(u16::MAX as u32) as u16,
            ))?;
            render(terminal)?;
        }
        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if data.contains(&b'q') || data.contains(&3) {
            session.close(channel)?;
        }
        Ok(())
    }
}

struct AppServer;

impl Server for AppServer {
    type Handler = Client;

    fn new_client(
        &mut self,
        _address: Option<std::net::SocketAddr>,
    ) -> Client {
        Client::default()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let key = russh::keys::load_secret_key("host.key", None)?;

    let config = server::Config {
        keys: vec![key],
        inactivity_timeout: Some(std::time::Duration::from_secs(300)),
        ..Default::default()
    };

    println!("Tuiminal SSH LP: 127.0.0.1:2222");

    AppServer
        .run_on_address(Arc::new(config), ("127.0.0.1", 2222))
        .await?;

    Ok(())
}

