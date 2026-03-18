# Contributing to Dust DDS

First off, thanks for taking the time to contribute!

If you are using Dust DDS and want to give back, we would appreciate to have you open GitHub issues and PRs for features, bugs and documentation improvements.

And if you like the project, but just don't have time to contribute, that's fine. There are other easy ways to support the project and show your appreciation, which we would also be very happy about:
- Sponsor us
- Star the project
- Share it on social media
- Refer this project in your project's readme
- Mention the project at local meetups and tell your friends/colleagues

## Code of Conduct

All contributors are expected to adhere to the following principles:

- **Be respectful**: Treat others with kindness and professionalism.
- **Follow best practices**: Write clean, maintainable code and provide clear documentation.
- **Report issues responsibly**: Use constructive feedback and provide enough detail for issues and pull requests.

## Legal Notice

When contributing to this project, you must agree that you have authored 100% of the content, that you have the necessary rights to the content and that the content you contribute may be provided under the project licence.

## How to Contribute

Contributions are made to this repo via Issues and Pull Requests (PRs). In general, we try our best to make sure issues/PRs are handled in a timely manner but, depending on the impact and our availability, it could take a while to get things solved. If an issue is urgent or has commercial value to you, you can always contact us on our [website](https://dust-dds.s2e-systems.com) and make use of our professional support services.

Before contributing search for existing Issues and PRs before creating your own. If there is no issue for your intended contribution, start by creating one. In this way can discuss the scope of the feature to be implemented. This helps to align the expectations and increases the likelihood that your contribution will be accepted once submitted.

### Issues

Issues should be used to report problems with the library, request a new feature, or to discuss potential changes before a PR is created. You can also open issues for questions if what you are trying to do is not explained in the documentation or examples.

If you find an Issue that addresses the problem you're having, please add a reaction or any additional reproduction information to the existing issue rather than creating a new one.

### Pull Requests

If you want to solve an open issue we generally welcome receiving PRs. PRs should fix only a single issue and they should contain all changes needed for code, tests, setup configuration and documentation. This doesn't mean that every PR should change all of it, but rather that they should be modified consistently and for the same underlying reason.

There are a few general guideline to observe when contributing:

1. **Refer the issue number on the title of your PR**: This makes it easy to create Release Notes and understand what all has been changed between Releases
1. **Do not add additional crate dependencies**: One of the goals of Dust DDS is to be used as a building block for other projects. This means we keep the number of dependent crates to a minimum. If you think a new dependency needs to be added this has to be discussed beforehand in the issue.
1. **Add tests for every new feature/bug**: Every new bug fix or added feature should come together with the relevant test cases. You can use the existing test suite as an example for creating new tests.
1. **Follow coding style and architecture**: All our code follows the standard Rust formatter style and is linted using clippy. This is automatically verified by PR checks.
1. **Make sure all PR checks are passing**: In general we will only start the review process once the automatic checks are all passing.
1. **Engage in discussion**: Be open to feedback and revise your contribution as needed.

Thank you for helping to improve this project!
