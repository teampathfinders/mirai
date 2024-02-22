# How to contribute

Thank you for taking the opportunity to the project! Your input is greatly appreciated. Feel free to contact the maintainers if you have anymore questions after reading this. 

### Writing bug reports
Bugs should be reported using GitHub's [issues](https://github.com/teampathfinders/mirai/issues). Upon creating a new issue, two default templates are presented to you which you can use. You can of course create your issue report without a template as well. Please make sure to report bugs related to security using the security policy instead of public issues.

Good bug reports tend to have:
1. A quick summary and/or background.
2. Steps to reproduce
2.a Be specific!
2.b Give sample code if you can. This should be easy to run for anyone that can compile the project.
3. What you expected would happen.
4. What actually happens.
5. Notes (possibly including why you think this might be happening, or stuff you tried that didn't work)

### Submitting changes
1. Fork the repository and create your branch from `master`.
2. If you feel like the code you wrote can and should be tested, make sure to write tests for it. 
3. Don't forget to add documentation to whatever code you have written. This makes sure that your code is easier to understand for other developers. 
4. Ensure that after you have made your changes, all of the tests pass. This is automatically checked by GitHub Actions when you create the pull request. 
5. Check that your code follows the style guidelines of this project. This is as easy as simply running `cargo fmt` on the project. The style configuration is specified in [rustfmt.toml](https://github.com/teampathfinders/mirai/blob/master/rustfmt.toml).

#### Licensing
By contributing, you agree that any contributions made will be under the same Apache 2.0 license that covers the rest of the Mirai project.
