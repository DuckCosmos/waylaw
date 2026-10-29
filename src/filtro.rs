use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct App {
    pub nome: String,
    pub icone: String,
    pub exec: String,
}

static APPS: OnceLock<Vec<App>> = OnceLock::new();

fn pastas() -> Vec<PathBuf> {
    let mut p = vec![PathBuf::from("/usr/share/applications")];
    if let Ok(home) = std::env::var("HOME") {
        p.push(PathBuf::from(home).join(".local/share/applications"));
    }
    p
}

fn icone_shell() -> String {
    let Ok(home) = std::env::var("HOME") else {
        return "utilities-terminal".to_string();
    };
    let caminho = format!("{home}/.local/share/waylaw/shell.png");
    if Path::new(&caminho).exists() {
        caminho
    } else {
        "utilities-terminal".to_string()
    }
}

fn ler_desktop(caminho: &Path) -> Option<App> {
    let conteudo = std::fs::read_to_string(caminho).ok()?;

    let mut nome = None;
    let mut icone = String::new();
    let mut exec = None;
    let mut esconder = false;
    let mut secao_certa = false;

    for linha in conteudo.lines() {
        let linha = linha.trim();

        if linha.starts_with('[') {
            secao_certa = linha == "[Desktop Entry]";
            continue;
        }
        if !secao_certa {
            continue;
        }

        if let Some(v) = linha.strip_prefix("Name=") {
            nome = Some(v.to_string());
        } else if let Some(v) = linha.strip_prefix("Icon=") {
            icone = v.to_string();
        } else if let Some(v) = linha.strip_prefix("Exec=") {
            exec = Some(v.to_string());
        } else if linha == "NoDisplay=true" || linha == "Hidden=true" {
            esconder = true;
        }
    }

    if esconder {
        return None;
    }

    Some(App {
        nome: nome?,
        icone,
        exec: exec?,
    })
}

fn carregar_apps() -> Vec<App> {
    let mut apps: Vec<App> = Vec::new();

    for pasta in pastas() {
        let Ok(entradas) = std::fs::read_dir(&pasta) else {
            continue;
        };
        for entrada in entradas.flatten() {
            let caminho = entrada.path();
            if caminho.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            if let Some(app) = ler_desktop(&caminho) {
                if !apps.iter().any(|a| a.nome == app.nome) {
                    apps.push(app);
                }
            }
        }
    }

    apps.sort_by(|a, b| a.nome.to_lowercase().cmp(&b.nome.to_lowercase()));
    apps
}

fn pastas_usuario() -> Vec<PathBuf> {
    let Ok(home) = std::env::var("HOME") else {
        return vec![];
    };
    let home = PathBuf::from(home);

    let mut dirs = Vec::new();

    if let Ok(txt) = std::fs::read_to_string(home.join(".config/user-dirs.dirs")) {
        for linha in txt.lines() {
            let linha = linha.trim();
            if linha.starts_with('#') {
                continue;
            }
            let Some((_, valor)) = linha.split_once('=') else {
                continue;
            };
            let valor = valor.trim().trim_matches('"');
            let expandido = valor.replace("$HOME", &home.to_string_lossy());
            let p = PathBuf::from(&expandido);
            if p.is_dir() {
                dirs.push(p);
            }
        }
    }

    if dirs.is_empty() {
        for nome in [
            "Desktop",
            "Documents",
            "Downloads",
            "Music",
            "Pictures",
            "Videos",
        ] {
            let p = home.join(nome);
            if p.is_dir() {
                dirs.push(p);
            }
        }
    }

    dirs
}

fn varrer(dir: &Path, nivel: u32, saida: &mut Vec<App>) {
    if nivel == 0 {
        return;
    }
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return;
    };

    for e in entradas.flatten() {
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_symlink() {
            continue;
        }

        let nome_arq = e.file_name();
        let nome_str = nome_arq.to_string_lossy();
        if nome_str.starts_with('.') {
            continue;
        }
        if matches!(
            nome_str.as_ref(),
            "node_modules" | "target" | "venv" | "__pycache__"
        ) {
            continue;
        }

        let caminho = e.path();
        if ft.is_dir() {
            varrer(&caminho, nivel - 1, saida);
        } else {
            saida.push(App {
                nome: nome_str.into_owned(),
                icone: "folder".to_string(),
                exec: format!(
                    "xdg-open '{}'",
                    caminho.to_string_lossy().replace('\'', "'\\''")
                ),
            });
        }
    }
}

fn carregar_arquivos() -> Vec<App> {
    let mut v = Vec::new();
    for d in pastas_usuario() {
        varrer(&d, 2, &mut v);
    }
    v
}

static ARQUIVOS: Mutex<Option<(Instant, Vec<App>)>> = Mutex::new(None);
const TTL: Duration = Duration::from_secs(30);

fn arquivos() -> Vec<App> {
    let mut guard = ARQUIVOS.lock().unwrap();
    if let Some((t, v)) = guard.as_ref() {
        if t.elapsed() < TTL {
            return v.clone();
        }
    }
    let novos = carregar_arquivos();
    *guard = Some((Instant::now(), novos.clone()));
    novos
}

fn arquivo_uso() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".local/share/waylaw_uso.txt"))
}

fn ler_uso() -> HashMap<String, u32> {
    let mut mapa = HashMap::new();
    if let Some(caminho) = arquivo_uso() {
        if let Ok(texto) = std::fs::read_to_string(caminho) {
            for linha in texto.lines() {
                if let Some((nome, n)) = linha.rsplit_once('\t') {
                    mapa.insert(nome.to_string(), n.parse().unwrap_or(0));
                }
            }
        }
    }
    mapa
}

pub fn calcular(texto: &str) -> Option<String> {
    let expr = texto
        .replace('x', "*")
        .replace('X', "*")
        .replace('×', "*")
        .replace('÷', "/")
        .replace(',', ".");

    let expr = expr.trim();

    if !expr.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    let tem_operador = expr.chars().any(|c| "+-*/^()".contains(c));

    let mut val = None;

    if tem_operador {
        if let Ok(v) = meval::eval_str(expr) {
            val = Some(v);
        } else {
            let partes: Vec<&str> = expr.split(|c| "+-*/^()".contains(c)).collect();
            if let Some(primeiro) = partes.first() {
                if let Ok(v) = primeiro.trim().parse::<f64>() {
                    val = Some(v);
                }
            }
        }
    } else {
        if let Ok(v) = expr.parse::<f64>() {
            val = Some(v);
        }
    }
    if let Some(v) = val {
        let resultado_str = if v.fract() == 0.0 {
            format!("{}", v as i64)
        } else {
            let s = format!("{}", v);
            if texto.contains(',') {
                s.replace('.', ",")
            } else {
                s
            }
        };
        Some(resultado_str)
    } else {
        None
    }
}
pub fn registrar_uso(nome: &str) {
    let mut uso = ler_uso();
    *uso.entry(nome.to_string()).or_insert(0) += 1;

    if let Some(caminho) = arquivo_uso() {
        let texto: String = uso.iter().map(|(n, c)| format!("{n}\t{c}\n")).collect();
        let _ = std::fs::write(caminho, texto);
    }
}

pub fn filtrar(texto: &str) -> Vec<App> {
    let busca = texto.to_lowercase();
    let uso = ler_uso();

    let mut res: Vec<App> = APPS
        .get_or_init(carregar_apps)
        .iter()
        .chain(arquivos().iter())
        .filter(|a| a.nome.to_lowercase().contains(&busca))
        .cloned()
        .collect();

    res.sort_by(|a, b| {
        let ua = uso.get(&a.nome).unwrap_or(&0);
        let ub = uso.get(&b.nome).unwrap_or(&0);
        ub.cmp(ua)
    });

    res.truncate(15);

    if let Some(texto_res) = calcular(texto) {
        // Ele insere a calculadora no topo exibindo o texto do jeito que o usuário digitou
        res.insert(
            0,
            App {
                nome: format!("{} = {}", texto.trim(), texto_res),
                icone: "accessories-calculator".to_string(),
                exec: format!("sh -c \"echo -n '{}' | wl-copy\"", texto_res), // Copia o resultado limpo
            },
        );
    } else if res.is_empty() && !texto.trim().is_empty() {
        res.push(App {
            nome: texto.to_string(),
            icone: icone_shell(),
            exec: texto.to_string(),
        });
    }

    res
}

pub fn abrir(nome: &str) {
    let exec = APPS
        .get_or_init(carregar_apps)
        .iter()
        .find(|a| a.nome == nome)
        .map(|a| a.exec.clone())
        .or_else(|| {
            arquivos()
                .into_iter()
                .find(|a| a.nome == nome)
                .map(|a| a.exec)
        })
        .unwrap_or_else(|| nome.to_string());

    let cmd: Vec<&str> = exec
        .split_whitespace()
        .filter(|p| !(p.starts_with('%') && p.len() == 2))
        .collect();

    let _ = Command::new("sh")
        .arg("-c")
        .arg(cmd.join(" "))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}
