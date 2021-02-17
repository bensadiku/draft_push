## draft_push

draft_push is a small script to update pull request status in Azure DevOps.


### Setup

1) Download binary [here](https://github.com/bensadiku/draft_push/releases/download/0.1/draft_push) & move it on the `~/` directory
2) Create a PAT, [see this](https://docs.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate?view=azure-devops&tabs=preview-page) p.s. don't give the token full access, it just needs `Read & write` on the `Code` sub section.
3) Once you have the token, run `./draft_push --config <tokenhere>`, and create the alias by executing `sh alias.sh`


Now you have the `xpush` alias, `git xpush` will mark the pull request as draft and publish it afterwards.