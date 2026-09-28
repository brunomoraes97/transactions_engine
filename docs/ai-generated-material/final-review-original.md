# Revisão final contra `notes.md`

Este documento descreve os cinco pontos encontrados na revisão final, o estado atual de cada um e uma proposta de solução.

## 1. Validação do cliente em `resolve` e `chargeback`

### Estado atual

Resolvido em `dispute`, `resolve` e `chargeback`.

Em `Engine::dispute`, a transação armazenada somente é utilizada quando:

```rust
transaction.client_id() == transaction_attempt.client_id()
    && !*transaction.in_dispute()
```

Existe também o teste unitário `dispute_is_ignored_when_transaction_belongs_to_another_client`.

`Engine::resolve` e `Engine::chargeback` agora fazem a mesma verificação de propriedade, além de verificarem se a transação está em disputa. Uma tentativa contendo o `tx` correto, mas um `client` diferente, é ignorada sem alterar a conta ou o estado da disputa.

### Solução implementada

Foi aplicada nos dois métodos a mesma verificação de propriedade já usada em `dispute`:

```rust
Some(transaction)
    if transaction.client_id() == transaction_attempt.client_id()
        && *transaction.in_dispute() =>
{
    (*transaction.client_id(), *transaction.amount())
}
```

Também foram adicionados dois testes unitários:

- `resolve_is_ignored_when_transaction_belongs_to_another_client`;
- `chargeback_is_ignored_when_transaction_belongs_to_another_client`.

Cada teste confirma que os saldos e o estado da disputa do proprietário permanecem iguais e que nenhuma conta é criada para o cliente incorreto.

## 2. Operações pendentes depois que uma conta é bloqueada

### Estado atual

`Account::deposit`, `Account::withdraw` e `Account::dispute` retornam `AccountError::AccountLocked` quando a conta está bloqueada. Existem testes para esses comportamentos.

`Account::resolve` e `Account::chargeback`, porém, não verificam `locked`. Portanto, se houver duas transações simultaneamente em disputa, o chargeback de uma delas bloqueia a conta, mas a outra ainda pode ser resolvida ou sofrer chargeback posteriormente.

Não existe atualmente um teste unitário para essa sequência.

### Decisão de negócio adotada

O enunciado diz que a conta deve ser imediatamente congelada depois de um chargeback, mas não detalha o que fazer com disputas que já estavam abertas. Há duas interpretações possíveis:

1. nenhuma operação pode alterar uma conta bloqueada, inclusive `resolve` e `chargeback` pendentes;
2. novas movimentações são proibidas, mas disputas que já estavam abertas ainda podem ser finalizadas.

Foi adotada a segunda interpretação: o bloqueio impede novas movimentações, mas `resolve` e `chargeback` podem concluir disputas que já estavam abertas. Essa decisão deve ser documentada posteriormente no `README.md`.

### Consequência para a implementação

Não se deve adicionar uma verificação de `locked` em `Account::resolve` ou `Account::chargeback`. Os testes futuros dessa regra devem demonstrar que depósitos, saques e novas disputas são rejeitados, enquanto disputas abertas antes do bloqueio ainda podem ser finalizadas.

## 3. Processamento do CSV em streaming

### Estado atual

Resolvido. `reader::process_transactions_from_csv` percorre `csv_reader.deserialize()` e envia cada `TransactionAttempt` diretamente para a engine.

O consumo de memória da entrada não cresce mais linearmente com o número de transações. A memória ainda cresce conforme o estado necessário das contas e das transações armazenadas, mas o conteúdo completo do arquivo não é mantido em um `Vec`.

### Como funciona o streaming

O `csv::Reader` já produz um iterador por meio de `deserialize()`. Não é necessário criar uma thread, usar código assíncrono ou implementar um iterador manualmente. Basta manter o reader aberto e entregar cada registro à engine assim que ele for desserializado:

```text
arquivo -> csv::Reader -> uma TransactionAttempt -> Engine::process
                         uma TransactionAttempt -> Engine::process
                         ...
```

Em qualquer momento, somente o reader, o estado acumulado das contas/transações e aproximadamente uma linha do CSV precisam estar na memória.

### Solução implementada

A solução moveu o laço para uma função que recebe a engine:

```rust
pub fn process_transactions_from_csv(
    engine: &mut Engine,
) -> Result<(), Box<dyn std::error::Error>> {
    // abre o arquivo e cria o csv_reader

    for transaction_result in csv_reader.deserialize() {
        let transaction: TransactionAttempt = transaction_result?;
        engine.process(transaction);
    }

    Ok(())
}
```

`main` agora chama essa função e não mantém um `Vec`. Uma alternativa arquiteturalmente mais desacoplada seria fazer `reader` devolver um iterador, mas isso introduziria tipos e lifetimes mais complexos sem oferecer benefício relevante para este projeto pequeno.

Para preservar a testabilidade, uma versão ainda melhor pode receber qualquer tipo que implemente `std::io::Read`; a abertura do caminho fica em uma função pequena, enquanto a leitura do CSV pode ser testada com bytes em memória.

## 4. Status de saída em caso de erro

### Problema anterior

O programa imprimia erros de leitura e escrita, mas executava `return` de uma função `main` que retornava `()`. Para o sistema operacional, isso representava término bem-sucedido, com exit code `0`.

Isso é importante em scripts e pipelines: o processo consumidor precisa distinguir uma execução válida de uma falha sem interpretar o texto escrito em `stderr`.

### Solução implementada

`main` agora retorna `std::process::ExitCode`:

- `ExitCode::FAILURE` depois de erro de leitura ou escrita;
- `ExitCode::SUCCESS` quando o processamento e a escrita terminam corretamente.

As mensagens continuam sendo enviadas para `stderr`, portanto não contaminam o CSV escrito em `stdout`.

## 5. Falhas de `cargo clippy` e `cargo fmt`

### Clippy

O aviso atual ocorre porque este método:

```rust
pub fn process(self: &mut Self, transaction_attempt: TransactionAttempt)
```

usa um tipo de receptor válido, mas desnecessariamente explícito. Em métodos comuns, a forma idiomática é:

```rust
pub fn process(&mut self, transaction_attempt: TransactionAttempt)
```

As duas assinaturas têm o mesmo comportamento e ownership. A segunda apenas usa a sintaxe convencional reconhecida pelo Clippy.

Depois da alteração, executar:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

`-D warnings` transforma todos os avisos em erros, sendo útil como verificação final ou em CI.

### Formatação

`cargo fmt --check` somente verifica e não modifica arquivos. Para aplicar a formatação automaticamente:

```bash
cargo fmt
```

Depois, confirmar:

```bash
cargo fmt --check
```

## Ordem recomendada

1. corrigir a propriedade do cliente em `resolve` e `chargeback` e adicionar os testes;
2. decidir e documentar a política para disputas abertas em contas bloqueadas;
3. implementar e testar essa política;
4. trocar a leitura baseada em `Vec` pelo processamento em streaming;
5. aplicar a correção simples do Clippy e executar `cargo fmt`;
6. executar todos os testes, Clippy e verificação de formatação.
