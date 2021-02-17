## draft_push

draft_push is a small script to update pull request status in Azure DevOps.


### Setup

1) Download binary [here](https://github.com/bensadiku/draft_push/releases/download/0.1/draft_push) & move it on the `~/` directory
2) Create a PAT, [see this](https://docs.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate?view=azure-devops&tabs=preview-page) p.s. don't give the token full access, it just needs `Read & write` on the `Code` sub section.
3) Once you have the token, run `./draft_push --config <tokenhere>`, and create the alias by executing `sh alias.sh`


Now you have the `xpush` alias, `git xpush` will mark the pull request as draft and publish it afterwards.


## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Licensing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.