# draft_push

draft_push is a post push script to update pull request status in Azure DevOps.


## Default setup

1) Download binary [here](https://github.com/bensadiku/draft_push/releases/download/0.2/draft_push) & move it on the `~/` directory
2) Create a PAT, [see this](https://docs.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate?view=azure-devops&tabs=preview-page) p.s. don't give the token full access, it just needs `Read & write` on the `Code` sub section.
3) Once you have the token, run `./draft_push -c <tokenhere>`. This will set the token and create a `xpush` alias with a default path `~/./draft_push`


Now you have the `xpush` alias, `git xpush` will mark the pull request as draft and publish it afterwards.


## Custom setup
###  ~ Configuring alias

You can configure the alias from the binary itself

e.g.

 `./draft_push -a '~/Desktop/./draft_push'`

 This alias assumes your draft_push binary is in the Desktop directory. Make sure your alias path and binary are in the same path.

### ~ Configuring token

Token can also be configured from the binary

e.g.

 `./draft_push -c <abc123tokenhere>`

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Licensing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.