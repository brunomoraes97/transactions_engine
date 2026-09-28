# Segunda revisão final contra `notes.md`

## Conclusão

O código atual está funcional de acordo com as regras descritas em `notes.md` e com a interpretação de negócio adotada para contas bloqueadas. Não foi encontrado um novo defeito funcional confirmado nesta revisão.

O projeto ainda não receberia a pontuação máxima porque não existe um `README.md` versionado explicando as decisões e suposições.

Estimativa atual: **9,8/10**. Com essa pendência resolvida, o projeto tem condições de se aproximar da pontuação máxima.

## Evidências executadas

### Compilação e testes

Foram executados:

```bash
cargo build
cargo test
```

Resultados:

- compilação concluída com sucesso;
- 32 testes unitários passando;
- 6 testes de integração passando;
- nenhum teste falhando.

### Formatação

Foi executado:

```bash
cargo fmt --check
```

Resultado: sucesso. O código está de acordo com o `rustfmt`.

### Clippy

Foi executado:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Resultado anterior: falha por um único aviso em `src/engine.rs`:

```rust
pub fn process(self: &mut Self, transaction_attempt: TransactionAttempt)
```

A forma idiomática equivalente é:

```rust
pub fn process(&mut self, transaction_attempt: TransactionAttempt)
```

Essa alteração foi aplicada. Ela não muda ownership, empréstimos ou comportamento; apenas utiliza a sintaxe convencional de receptor de método.

Resultado atual: o Clippy passa sem avisos, inclusive com `-D warnings`.

### Execução extensa

O programa processou `personal/test-extensive.csv`:

- 500 transações de entrada;
- exit code `0`;
- nenhum conteúdo em `stderr`;
- 110 contas no resultado;
- nenhuma violação de `total == available + held`.

O arquivo extenso está em `personal/`, que não está versionado. Portanto, ele ajuda na validação local, mas não estará disponível para um avaliador que clonar o repositório.

### Falha de entrada

O binário também foi executado sem o argumento do CSV:

- exit code `1`;
- nenhum conteúdo em `stdout`;
- mensagem de erro somente em `stderr`.

Isso confirma que uma falha não produz um CSV parcial em `stdout` e é comunicada corretamente ao sistema operacional.

## Revisão por critério

### 1. Basics

**Estado: atendido.**

- o projeto compila com `cargo build`;
- pode ser executado com `cargo run -- arquivo.csv`;
- lê o CSV indicado pelo primeiro argumento;
- escreve somente o resultado CSV em `stdout` durante uma execução bem-sucedida;
- escreve diagnósticos em `stderr`;
- retorna exit code diferente de zero em falhas de leitura ou escrita;
- `cargo fmt --check` passa.

Melhoria pequena: o programa utiliza o primeiro argumento, mas não rejeita argumentos adicionais. Como o enunciado diz que o arquivo é o primeiro e único argumento, essa validação poderia ser acrescentada, embora não afete o fluxo normal esperado.

### 2. Completeness

**Estado: atendido.**

Estão implementados:

- depósito;
- saque;
- disputa;
- resolução;
- chargeback;
- bloqueio após chargeback;
- rejeição de movimentações novas em conta bloqueada;
- consulta de transações anteriores por `tx`;
- validação de que disputa, resolução e chargeback pertencem ao cliente informado;
- rejeição de resolução e chargeback quando a transação não está em disputa.

Os eventos sem valor utilizam `amount: Option<Decimal>`. Para depósito ou saque sem valor, `TransactionAttempt::amount()` fornece zero e a operação é rejeitada como valor inválido.

### 3. Correctness

**Estado: atendido.**

As movimentações preservam as relações esperadas:

- depósito aumenta `available` e `total`;
- saque reduz `available` e `total` somente quando existem fundos suficientes;
- disputa reduz `available`, aumenta `held` e preserva `total`;
- resolução reduz `held`, aumenta `available` e preserva `total`;
- chargeback reduz `held` e `total` e bloqueia a conta.

As verificações de propriedade agora existem em `dispute`, `resolve` e `chargeback`. Os testes unitários confirmam que outro cliente não consegue atuar sobre a transação do proprietário.

#### Decisão sobre contas bloqueadas

Foi adotada esta interpretação:

> O bloqueio rejeita novas movimentações, mas permite concluir disputas que já estavam abertas antes do chargeback.

Por isso, `Account::resolve` e `Account::chargeback` intencionalmente não verificam `locked`. Isso não deve ser tratado como defeito, pois é uma decisão consciente para uma ambiguidade do enunciado.

O teste `pending_disputes_can_be_completed_after_account_is_locked` torna essa decisão executável e evita que uma manutenção futura adicione uma verificação de bloqueio nesses métodos por engano. O cenário testado:

1. realiza três depósitos para o mesmo cliente;
2. coloca todos em disputa;
3. realizar chargeback do primeiro, bloqueando a conta;
4. resolve o segundo e realiza chargeback do terceiro;
5. confirma que ambas as disputas pendentes foram concluídas mesmo com a conta bloqueada.

#### Suposição sobre saques disputados

O código armazena depósitos e saques bem-sucedidos e permite que ambos sejam referenciados por uma disputa. A fórmula aplicada é exatamente a fórmula geral descrita pelo enunciado: reduzir `available` e aumentar `held` pelo valor da transação.

O enunciado não define um comportamento diferente para a disputa de um saque. Essa é outra suposição que deve ser declarada no README para evitar interpretação implícita.

### 4. Safety and robustness

**Estado: atendido.**

- valores monetários utilizam `rust_decimal::Decimal`, evitando erros típicos de ponto flutuante;
- IDs usam `u16` e `u32`, conforme pedido;
- fundos insuficientes não alteram a conta;
- valores não positivos são rejeitados;
- uma segunda disputa simultânea da mesma transação é ignorada;
- eventos referentes a transações inexistentes ou em estado incompatível são ignorados;
- eventos de cliente incorreto são ignorados;
- erros estruturais de leitura e escrita encerram o processo com falha;
- erros não são misturados ao CSV de saída.

O silêncio para falhas de negócio é coerente com o enunciado, que permite ignorar eventos inválidos. Não imprimir um erro por linha também evita corromper `stdout` ou produzir grande volume de logs em arquivos extensos.

#### Testes de robustez

Foram adicionados testes de integração para:

- execução sem argumento, esperando exit code diferente de zero;
- caminho inexistente, esperando exit code diferente de zero;
- CSV estruturalmente inválido, esperando exit code diferente de zero;
- confirmação de que `stdout` permanece vazio nesses casos.

Esses testes confirmam o exit code de falha, a ausência de conteúdo em `stdout` e a presença do diagnóstico em `stderr`.

### 5. Efficiency

**Estado: atendido.**

`reader::process_transactions_from_csv` percorre `csv_reader.deserialize()` e envia cada transação diretamente para `Engine::process`. O arquivo completo não é mais acumulado em um `Vec<TransactionAttempt>`.

Em relação à entrada, a memória permanece aproximadamente constante: existe apenas a transação atual além dos buffers internos do leitor.

O `State` ainda cresce com:

- as contas encontradas;
- as transações bem-sucedidas que podem ser referenciadas posteriormente por uma disputa.

Esse crescimento é necessário para as regras atuais, pois uma disputa pode apontar para qualquer `tx` processado anteriormente.

#### Observação arquitetural

O módulo `reader` conhece diretamente `Engine`. Isso mantém a implementação simples, mas acopla parsing e processamento. Se o projeto crescer, uma evolução possível seria separar:

- uma função que recebe qualquer `std::io::Read` e produz eventos;
- a abertura do arquivo indicado nos argumentos;
- o processamento dos eventos pela engine.

Não é uma correção necessária para o desafio atual.

### 6. Maintainability

**Estado: bom, com pendências claras.**

Pontos positivos:

- responsabilidades principais estão separadas entre `Account`, `State`, `Engine`, `reader` e `output`;
- a lógica de saldo está concentrada em `Account`;
- a lógica de coordenação e validação de transações está em `Engine`;
- o registro de contas e transações está encapsulado em `State`;
- os testes usam Arrange, Act e Assert;
- casos felizes e vários modos de falha possuem cobertura unitária;
- há testes de integração executando o binário real.

Pendência: criar um `README.md` versionado.

## README necessário

Não existe atualmente um `README.md` versionado. Como `notes.md` está dentro de `personal/` e essa pasta não é versionada, o avaliador não verá as decisões registradas somente ali.

O README futuro deve conter, no mínimo:

1. objetivo do projeto;
2. comandos para compilar, executar e testar;
3. exemplo de entrada e saída;
4. resumo da arquitetura;
5. política de erros técnicos e eventos de negócio inválidos;
6. decisão de permitir finalizar disputas já abertas após o bloqueio da conta;
7. suposição adotada para disputas que referenciam saques;
8. explicação de que o input é processado em streaming;
9. relação dos testes e cenários importantes cobertos.

## Pontuação estimada

| Critério | Estimativa | Justificativa |
|---|---:|---|
| Basics | 2,0 / 2,0 | Compila, executa, lê e escreve corretamente; formatação passa. |
| Completeness | 2,0 / 2,0 | Todos os tipos de evento foram implementados. |
| Correctness | 2,0 / 2,0 | Regras principais, propriedade do cliente e política pós-bloqueio estão testadas. |
| Safety and robustness | 2,0 / 2,0 | Erros, eventos inválidos e falhas técnicas do processo estão cobertos. |
| Efficiency | 1,0 / 1,0 | Entrada processada em streaming. |
| Maintainability | 0,8 / 1,0 | Boa separação, testes e Clippy verde; ainda falta o README. |
| **Total** | **9,8 / 10,0** | Projeto funcional; a principal pendência restante é a documentação versionada. |

Essa pontuação é uma estimativa, pois o enunciado não define pesos numéricos oficiais para cada item.

## Ordem recomendada para alcançar a versão final

1. ~~trocar `self: &mut Self` por `&mut self` em `Engine::process`;~~
2. ~~executar Clippy novamente e confirmar que não existem avisos;~~
3. ~~adicionar o teste que formaliza a política de disputas pendentes depois do bloqueio;~~
4. ~~adicionar testes de integração para os exit codes de falha;~~
5. criar e versionar o README com as decisões listadas acima;
6. ~~executar `cargo test`, `cargo fmt --check` e Clippy como verificação final.~~
