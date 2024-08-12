#!/usr/bin/python
#################################################################
# Use and redistribution is source and binary forms is permitted
# subject to the OMG-DDS INTEROPERABILITY TESTING LICENSE found
# at the following URL:
#
# https://github.com/omg-dds/dds-rtps/blob/master/LICENSE.md
#
#################################################################

import importlib
import time
import re
import pexpect
import argparse
import junitparser
import multiprocessing
from datetime import datetime
import tempfile
from os.path import exists
import inspect

from rtps_test_utilities import ReturnCode, log_message, no_check, remove_ansi_colors

# This parameter is used to save the samples the Publisher sends.
# MAX_SAMPLES_SAVED is the maximum number of samples saved.
MAX_SAMPLES_SAVED = 500

def run_subscriber_shape_main(
        name_executable: str,
        parameters: str,
        produced_code: "list[int]",
        produced_code_index: int,
        subscriber_index: int,
        samples_sent: "list[multiprocessing.Queue]",
        verbosity: bool,
        timeout: int,
        file: tempfile.TemporaryFile,
        subscriber_finished: multiprocessing.Event,
        publishers_finished: "list[multiprocessing.Event]",
        check_function: "function"):

    """ This function runs the subscriber shape_main application with
        the specified parameters. Then it saves the
        return code in the variable produced_code.

        name_executable <<in>>: name of the shape_main application to run
                as a Subscriber.
        parameters <<in>>: shape_main application parameter list.
        produced_code <<out>>: this variable will be overwritten with
                the obtained ReturnCode.
        produced_code_index <<in>>: index of the produced_code list
                where the ReturnCode is saved.
        subscriber_index <<in>>: index of the subscriber. For the first
                subscriber it is 1, for the second 2, etc.
        samples_sent <<in>>: list of multiprocessing Queues with the samples
                the Publishers send. Element 1 of the list is for
                Publisher 1, etc.
        verbosity <<in>>: print debug information.
        timeout <<in>>: time pexpect waits until it matches a pattern.
        file <<inout>>: temporal file to save shape_main application output.
        subscriber_finished <<inout>>: object event from multiprocessing
                that is set when the subscriber is finished.
        publishers_finished <<inout>>: list of events from multiprocessing
                that are set when the publishers are finished.
                Element 1 of the list is for Publisher 1, etc.
        check_function <<in>>: function to check how the samples are received
                by the Subscriber. By default it does not check anything.

        The function runs the shape_main application as a Subscriber
        with the parameters defined.
        The Subscriber shape_main application follows the next steps:
            * The topic is created.
            * The Data Reader is created.
            * The Data Reader matches with a Data Writer.
            * The Data Reader detects the Data Writer as alive.
            * The Data Reader receives data.

        If the shape_main application passes one step, it prints a specific
        string pattern. This function matches that pattern and and waits
        for the next input string from the shape_main application. If the
        shape_main application stops at some step, it prints an error message.
        When this function matches an error string (or doesn't match
        an expected pattern in the specified timeout),
        the corresponding ReturnCode is saved in
        produced_code[produced_code_index] and the process finishes.
    """

    # Step 1: run the executable
    log_message(f'Running shape_main application Subscriber {subscriber_index}',
            verbosity)
    child_sub = pexpect.spawnu(f'{name_executable} {parameters}')
    child_sub.logfile = file

    # Step 2: Check if the topic is created
    log_message(f'Subscriber {subscriber_index}: Waiting for topic creation',
            verbosity)
    index = child_sub.expect(
        [
            'Create topic:', # index = 0
            pexpect.TIMEOUT, # index = 1
            pexpect.EOF # index = 2
        ],
        timeout
    )

    if index == 1 or index == 2:
        produced_code[produced_code_index] = ReturnCode.TOPIC_NOT_CREATED
    elif index == 0:
        # Step 3: Check if the reader is created
        log_message(f'Subscriber {subscriber_index}: Waiting for DataReader '
                'creation', verbosity)
        index = child_sub.expect(
            [
                'Create reader for topic:', # index = 0
                pexpect.TIMEOUT, # index = 1
                'failed to create content filtered topic' # index = 2
            ],
            timeout
        )

        if index == 1:
            produced_code[produced_code_index] = ReturnCode.READER_NOT_CREATED
        elif index == 2:
            produced_code[produced_code_index] = ReturnCode.FILTER_NOT_CREATED
        elif index == 0:
            # Step 4: Check if the reader matches the writer
            log_message(f'Subscriber {subscriber_index}: Waiting for matching '
                    'DataWriter', verbosity)
            # the normal flow of the application is to find on_subscription_matched()
            # and then on_liveliness_changed(). However, in some cases the
            # situation is the opposite. This will handle those cases.
            # pexpect searches the results in order, so a matching on
            # on_subscription_matched() takes precedence over on_liveliness_changed()
            index = child_sub.expect(
                [
                    '\[[0-9]+\]', # index = 0
                    'on_requested_incompatible_qos()', # index = 1
                    'on_requested_deadline_missed()', # index = 2
                    pexpect.TIMEOUT, # index = 3
                ],
                timeout
            )

            if index == 1:
                produced_code[produced_code_index] = ReturnCode.INCOMPATIBLE_QOS
            elif index == 2:
                produced_code[produced_code_index] = ReturnCode.DEADLINE_MISSED
            elif index == 3:
                produced_code[produced_code_index] = ReturnCode.DATA_NOT_RECEIVED
            elif index == 0:
                # Step 5: Receiving samples
                log_message(f'Subscriber {subscriber_index}: Receiving samples',
                    verbosity)

                # this is used to check how the samples are arriving
                # to the Subscriber. By default it does not check
                # anything and returns ReturnCode.OK.
                produced_code[produced_code_index] = check_function(
                    child_sub, samples_sent, timeout)

    subscriber_finished.set()   # set subscriber as finished
    log_message(f'Subscriber {subscriber_index}: Waiting for Publishers to '
            'finish', verbosity)
    for element in publishers_finished:
        element.wait()   # wait for all publishers to finish
    # Send SIGINT to nicely close the application
    child_sub.sendintr()
    return


def run_publisher_shape_main(
        name_executable: str,
        parameters: str,
        produced_code: "list[int]",
        produced_code_index: int,
        publisher_index: int,
        samples_sent: multiprocessing.Queue,
        verbosity: bool,
        timeout: int,
        file: tempfile.TemporaryFile,
        subscribers_finished: "list[multiprocessing.Event]",
        publisher_finished: multiprocessing.Event):

    """ This function runs the publisher shape_main application with
        the specified parameters. Then it saves the
        return code in the variable produced_code.

        name_executable: <<in>> name of the shape_main application to run
                as a Publisher.
        parameters <<in>>: shape_main application parameter list.
        produced_code <<out>>: this variable will be overwritten with
                the obtained ReturnCode.
        produced_code_index <<in>>: index of the produced_code list
                where the ReturnCode is saved.
        publisher_index <<in>>: index of the publisher. For the first
                publisher it is 1, for the second 2, etc.
        samples_sent <<out>>: this variable contains the samples
                the Publisher sends.
        verbosity <<in>>: print debug information.
        timeout <<in>>: time pexpect waits until it matches a pattern.
        file <<inout>>: temporal file to save shape_main application output.
        subscribers_finished <<inout>>: list of events from multiprocessing
                that are set when the subscribers are finished.
                Element 1 of the list is for Subscriber 1, etc.
        publisher_finished <<inout>>: object event from multiprocessing
                that is set when the publisher is finished.

        The function runs the shape_main application as a Publisher
        with the parameters defined.
        The Publisher shape_main application follows the next steps:
            * The topic is created.
            * The Data Writer is created.
            * The Data Writer matches with a Data Reader.
            * The Data Writer sends data.

        If the shape_main application passes one step, it prints a specific
        string pattern. This function matches that pattern and and waits
        for the next input string from the shape_main application. If the
        shape_main application stops at some step, it prints an error message.
        When this function matches an error string (or doesn't match
        an expected pattern in the specified timeout),
        the corresponding ReturnCode is saved in
        produced_code[produced_code_index] and the process finishes.
    """

    # Step 1: run the executable
    log_message(f'Running shape_main application Publisher {publisher_index}',
            verbosity)
    child_pub = pexpect.spawnu(f'{name_executable} {parameters}')
    child_pub.logfile = file

    # Step 2: Check if the topic is created
    log_message(f'Publisher {publisher_index}: Waiting for topic creation',
            verbosity)
    index = child_pub.expect(
        [
            'Create topic:', # index == 0
            pexpect.TIMEOUT, # index == 1
            pexpect.EOF # index == 2
        ],
        timeout
    )

    if index == 1 or index == 2:
        produced_code[produced_code_index] = ReturnCode.TOPIC_NOT_CREATED
    elif index == 0:
        # Step 3: Check if the writer is created
        log_message(f'Publisher {publisher_index}: Waiting for DataWriter '
                'creation', verbosity)
        index = child_pub.expect(
            [
                'Create writer for topic', # index = 0
                pexpect.TIMEOUT # index = 1
            ],
            timeout
        )
        if index == 1:
            produced_code[produced_code_index] = ReturnCode.WRITER_NOT_CREATED
        elif index == 0:
            # Step 4: Check if the writer matches the reader
            log_message(f'Publisher {publisher_index}: Waiting for matching '
                    'DataReader', verbosity)
            index = child_pub.expect(
                [
                    'on_publication_matched()', # index = 0
                    pexpect.TIMEOUT, # index = 1
                    'on_offered_incompatible_qos' # index = 2
                ],
                timeout
            )
            if index == 1:
                produced_code[produced_code_index] = ReturnCode.READER_NOT_MATCHED
            elif index == 2:
                produced_code[produced_code_index] = ReturnCode.INCOMPATIBLE_QOS
            elif index == 0:
                # In the case that the option -w is selected, the Publisher
                # saves the samples sent in order, so the Subscriber can check
                # them. In this way, the script can check some functionality
                # such as reliability.
                # In the case that the option -w is not selected, the Publisher
                # will only save the ReturnCode OK.
                if '-w' in parameters:
                    # Step 5: Check whether the writer sends the samples
                    index = child_pub.expect([
                            '\[[0-9]+\]', # index = 0
                            'on_offered_deadline_missed()', # index = 1
                            pexpect.TIMEOUT # index = 2
                        ],
                        timeout)
                    if index == 1:
                        produced_code[produced_code_index] = ReturnCode.DEADLINE_MISSED
                    elif index == 2:
                        produced_code[produced_code_index] = ReturnCode.DATA_NOT_SENT
                    elif index == 0:
                        produced_code[produced_code_index] = ReturnCode.OK
                        log_message(f'Publisher {publisher_index}: Sending '
                                'samples', verbosity)
                        for x in range(0, MAX_SAMPLES_SAVED, 1):
                            # At this point, at least one sample has been printed
                            # Therefore, that sample is added to samples_sent.
                            pub_string = re.search('[0-9]+ [0-9]+ \[[0-9]+\]',
                                    child_pub.before + child_pub.after)
                            samples_sent.put(pub_string.group(0))
                            index = child_pub.expect([
                                    '\[[0-9]+\]', # index = 0
                                    'on_offered_deadline_missed()', # index = 1
                                    pexpect.TIMEOUT # index = 2
                                ],
                                timeout)
                            if index == 1:
                                produced_code[produced_code_index] = ReturnCode.DEADLINE_MISSED
                                break
                            elif index == 2:
                                produced_code[produced_code_index] = ReturnCode.DATA_NOT_SENT
                                break
                else:
                    produced_code[produced_code_index] = ReturnCode.OK

    log_message(f'Publisher {publisher_index}: Waiting for Subscribers to finish',
            verbosity)
    for element in subscribers_finished:
        element.wait() # wait for all subscribers to finish
    publisher_finished.set()   # set publisher as finished
    # Send SIGINT to nicely close the application
    child_pub.sendintr()
    return


def run_test(
    name_executable_pub:str,
    name_executable_sub:str,
    test_case: junitparser.TestCase,
    parameters: "list[str]",
    expected_codes: "list[str]",
    verbosity: bool,
    timeout: int,
    check_function: "function"):

    """ Run the Publisher and the Subscriber applications and check
        the actual and the expected ReturnCode.

        name_executable_pub <<in>>: name of the shape_main application to run
                as a Publisher.
        name_executable_sub <<in>>: name of the shape_main application to run
                as a Subscriber.
        test_case <<inout>>: testCase object to test.
        parameters <<in>>: list of shape_main application parameters.
        expected_codes <<in>>: list of ReturnCodes the Publishers and
                the Subscribers would obtain in a non error situation.
        verbosity <<in>>: print debug information.
        timeout <<in>>: time pexpect waits until it matches a pattern.
        check_function <<in>>: function to check how the samples are received
                by the Subscriber. By default it does not check anything.

        The function runs several different processes: one for each Publisher
        and one for each Subscriber shape_main application.
        The number of processes depends on how many elements are in
        the list of parameters.
        Then it checks that the codes obtained are the expected ones.
    """

    log_message(f'run_test parameters:\n'
            f'    name_executable_pub: {name_executable_pub}\n'
            f'    name_executable_sub: {name_executable_sub}\n'
            f'    test_case: {test_case.name}\n'
            f'    parameters: {parameters}\n'
            f'    expected_codes: {expected_codes}\n'
            f'    verbosity: {verbosity}\n'
            f'    timeout: {timeout}\n'
            f'    check_function: {check_function.__name__}',
            verbosity)

    # numbers of publishers/subscriber we will have. It depends on how
    # many strings of parameters we have.
    num_entities = len(parameters)

    # Manager is a shared memory section where all processes can access.
    # 'return_codes' is a list of elements where the different processes
    # (publishers and subscribers shape_main applications) copy their ReturnCode.
    # These ReturnCodes are identified by the index within the list,
    # every index identifies one shape_main application. Therefore, only one
    # shape_main application must modify one element of the list.
    # Once all processes are finished, the list 'return_codes' contains
    # the ReturnCode in the corresponding index. This index is set manually
    # and we need it in order to use it later.
    # Example: (1 Publisher and 1 Subscriber)
    #   Processes:
    #     - Publisher Process (index = 0)
    #     - Subscriber Process (index = 1)
    #   Code contains:
    #     - return_codes[0] contains Publisher shape_main application ReturnCode
    #     - return_codes[1] contains Subscriber shape_main application ReturnCode
    manager = multiprocessing.Manager()
    return_codes = manager.list(range(num_entities))
    samples_sent = [] # used for storing the samples the Publishers send.
                      # It is a list with one Queue for each Publisher.

    # list of multiprocessing Events used as semaphores to control the end of
    # the processes, one for each entity.
    subscribers_finished = []
    publishers_finished = []
    publisher_number = 0
    subscriber_number = 0
    # entity_type defines the name of the entity: Publisher/Subscriber_<number>.
    entity_type = []
    # list of files to save the shape_main output, one for each entity.
    temporary_file = []
    # list of shape_main application outputs, one for each entity.
    shape_main_application_output = []
    # list of processes, one for each entity
    entity_process = []
    # list of shape_main application outputs, edited to use in the html code.
    shape_main_application_output_edited = []
    # Create these elements earlier because they are needed
    # to define the processes.
    for element in parameters:
        temporary_file.append(tempfile.TemporaryFile(mode='w+t'))
        if ('-P ' in element or element.endswith('-P')):
            publishers_finished.append(multiprocessing.Event())
            samples_sent.append(multiprocessing.Queue())
        elif ('-S ' in element or element.endswith('-S')):
            subscribers_finished.append(multiprocessing.Event())
        else:
            raise RuntimeError('Error in the definition of shape_main '
                'application parameters. Neither Publisher or Subscriber '
                'defined.')

    # Create and run the processes for the different shape_main applications
    for i in range(0, num_entities):
        if ('-P ' in parameters[i] or parameters[i].endswith('-P')):
            entity_process.append(multiprocessing.Process(
                    target=run_publisher_shape_main,
                    kwargs={
                        'name_executable':name_executable_pub,
                        'parameters':parameters[i],
                        'produced_code':return_codes,
                        'produced_code_index':i,
                        'publisher_index':publisher_number+1,
                        'samples_sent':samples_sent[publisher_number],
                        'verbosity':verbosity,
                        'timeout':timeout,
                        'file':temporary_file[i],
                        'subscribers_finished':subscribers_finished,
                        'publisher_finished':publishers_finished[publisher_number]}))
            publisher_number += 1
            entity_type.append(f'Publisher_{publisher_number}')
            time.sleep(1)

        elif('-S ' in parameters[i] or parameters[i].endswith('-S')):
            # Wait 1 second before running the subscriber to avoid conflicts between
            # the programs on startup
            time.sleep(1)

            entity_process.append(multiprocessing.Process(
                    target=run_subscriber_shape_main,
                    kwargs={
                        'name_executable':name_executable_sub,
                        'parameters':parameters[i],
                        'produced_code':return_codes,
                        'produced_code_index':i,
                        'subscriber_index':subscriber_number+1,
                        'samples_sent':samples_sent,
                        'verbosity':verbosity,
                        'timeout':timeout,
                        'file':temporary_file[i],
                        'subscriber_finished':subscribers_finished[subscriber_number],
                        'publishers_finished':publishers_finished,
                        'check_function':check_function}))
            subscriber_number += 1
            entity_type.append(f'Subscriber_{subscriber_number}')
        else:
            raise RuntimeError('Error in the definition of shape_main '
                'application parameters. Neither Publisher or Subscriber '
                'defined.')

        entity_process[i].start()

    for element in entity_process:
        element.join()     # Wait until the processes finish

    log_message('Reading shape_main application console output from '
                'temporary files',
                verbosity)
    for element in temporary_file:
        element.seek(0)
        shape_main_application_output.append(element.read())

    # create an attribute for each entity that will contain their parameters
    for i in range(0, num_entities):
        junitparser.TestCase.i = junitparser.Attr(entity_type[i])
        test_case.i = parameters[i]

    # code[i] contains publisher/subscriber i shape_main application ReturnCode,
    # If we have 1 Publisher (index 0) and 1 Subscriber (index 1):
    # code[0] will contain entity 0 ReturnCode -> Publisher Return Code
    # code[1] will contain entity 1 ReturnCode -> Subscriber Return Code
    # The order of the entities will depend on the definition of the parameters.
    test_result_correct = True
    for i in range(0, num_entities):
        if expected_codes[i] != return_codes[i]:
            # if any of the ReturnCode does not match with the expected code,
            # there is an error.
            test_result_correct = False

    if test_result_correct:
        print(f'{test_case.name} : OK')

    else:
        print(f'{test_case.name} : ERROR')
        for i in range(0, num_entities):
            print(f'{entity_type[i]} expected code: {expected_codes[i].name}; '
                f'Code found: {return_codes[i].name}')

            log_message(f'\nInformation about {entity_type[i]}:\n '
                      f'{shape_main_application_output[i]} ', verbosity)

            # Change the '\n' and SIGINT chars to html <br>
            shape_main_application_output_edited.append(
                        shape_main_application_output[i]
                        .replace('\n', '<br>')
                        .replace(chr(3),'<br>'))

        # generate the table for the html code.
        message = \
            '<table> ' \
                '<tr> ' \
                    '<th/> ' \
                    '<th> Expected Code </th> ' \
                    '<th> Code Produced </th> ' \
                '</tr> '
        for i in range(num_entities):
            message += \
                '<tr> ' \
                    f'<th> {entity_type[i]} </th> ' \
                    f'<th> {expected_codes[i].name} </th> ' \
                    f'<th> {return_codes[i].name} </th> ' \
                '</tr>'
        message += '</table>'
        for i in range(0, num_entities):
            message += f'<strong> Information {entity_type[i]} </strong>' \
                    f'<br> {shape_main_application_output_edited[i]} <br>'
        message = remove_ansi_colors(message)
        test_case.result = [junitparser.Failure(message)]

    for element in temporary_file:
        element.close()

class Arguments:
    def parser():
        parser = argparse.ArgumentParser(
            description='Validation of interoperability of products compliant '
                'with OMG DDS-RTPS standard. This script generates automatically '
                'the verification between two shape_main executables. '
                'It also generates an XML report in JUnit format.',
            add_help=True)

        gen_opts = parser.add_argument_group(title='general options')
        gen_opts.add_argument('-P', '--publisher',
            default=None,
            required=True,
            type=str,
            metavar='publisher_executable_name',
            help='Path to the Publisher shape_main application. '
                'It may be absolute or relative path. Example: if the executable '
                'is in the same folder as the script: '
                '"-P ./rti_connext_dds-6.1.1_shape_main_linux".')
        gen_opts.add_argument('-S', '--subscriber',
            default=None,
            required=True,
            type=str,
            metavar='subscriber_executable_name',
            help='Path to the Subscriber shape_main application. '
                'It may be absolute or relative path. Example: if the executable '
                'is in the same folder as the script: '
                '"-S ./rti_connext_dds-6.1.1_shape_main_linux".')

        optional = parser.add_argument_group(title='optional parameters')
        optional.add_argument('-v','--verbose',
            default=False,
            required=False,
            action='store_true',
            help='Print debug information to stdout. This option also shows the '
                'shape_main application output in case of error. '
                'If this option is not used, only the test results are printed '
                'in the stdout. (Default: False).')
        optional.add_argument('-x','--data-representation',
            default="2",
            required=None,
            type=str,
            choices=["1","2"],
            help='Data Representation used if no provided when running the '
                'shape_main application. If this application already sets the '
                'data representation, this parameter is not used.'
                'The potential values are 1 for XCDR1 and 2 for XCDR2.'
                'Default value 2.')

        tests = parser.add_argument_group(title='Test Case and Test Suite')
        tests.add_argument('-s', '--suite',
            default='test_suite',
            required=False,
            metavar='test_suite_dictionary_file',
            type=str,
            help='Test Suite that is going to be tested. '
                'Test Suite is a file with a Python dictionary defined. It must '
                'be located on the same directory as interoperability_report. '
                'This value should not contain the extension ".py", '
                'only the name of the file. '
                'It will run all the dictionaries defined in the file. '
                '(Default: test_suite).')

        enable_disable = tests.add_mutually_exclusive_group(required=False)
        enable_disable.add_argument('-t', '--test',
            nargs='+',
            default=None,
            required=False,
            type=str,
            metavar='test_cases',
            help='Test Case that the script will run. '
                'This option is not supported with --disable-test. '
                'This allows to set multiple values separated by a space. '
                '(Default: run all Test Cases from the Test Suite.)')
        enable_disable.add_argument('-d', '--disable-test',
            nargs='+',
            default=None,
            required=False,
            type=str,
            metavar='test_cases_disabled',
            help='Test Case that the script will skip. '
                'This allows to set multiple values separated by a space. '
                'This option is not supported with --test. (Default: None)')

        out_opts = parser.add_argument_group(title='output options')
        out_opts.add_argument('-o', '--output-name',
            required=False,
            metavar='filename',
            type=str,
            help='Name of the xml report that will be generated. '
                'If the file passed already exists, it will add '
                'the new results to it. In other case it will create '
                'a new file. '
                '(Default: <publisher_name>-<subscriber_name>-date.xml)')

        return parser

# this function checks if the test case exist in the test suite
def are_tests_in_test_suite(test_suite, suite_name, test_cases):
    all_test_cases_exist = True
    if test_cases != None:
        for i in test_cases:
            if i not in test_suite:
                print(f'Test Case <{i}> not contained in Test Suite '
                        f'<{suite_name}>.')
                all_test_cases_exist = False
    return all_test_cases_exist

def main():
    parser = Arguments.parser()
    args = parser.parse_args()

    options = {
        'publisher': args.publisher,
        'subscriber': args.subscriber,
        'verbosity': args.verbose,
        'test_suite': args.suite,
        'test_cases': args.test,
        'test_cases_disabled': args.disable_test,
        'data_representation': args.data_representation,
    }

    # The executables's names are supposed to follow the pattern: name_shape_main
    # We will keep only the part of the name that is useful, deleting the path
    # and the substring '_shape_main'.
    # Example: if the shape_main application's name (including the path) is:
    #  ./srcCxx/objs/x64Linux4gcc7.3.0/rti_connext_dds-6.1.1_shape_main_linux
    # we will take the substring rti_connext_dds-6.1.1.
    # That will be the name that will appear in the report.
    name_publisher = options['publisher'].split('_shape')[0].split('-shape')[0].split('/')[-1]
    name_subscriber = options['subscriber'].split('_shape')[0].split('-shape')[0].split('/')[-1]

    if args.output_name is None:
        now = datetime.now()
        date_time = now.strftime('%Y%m%d-%H_%M_%S')
        options['filename_report'] = \
            f'{name_publisher}-{name_subscriber}-{date_time}.xml'
        xml = junitparser.JUnitXml()

    else:
        options['filename_report'] = args.output_name
        file_exists = exists(options['filename_report'])
        if file_exists:
            xml = junitparser.JUnitXml.fromfile(options['filename_report'])
        else:
            xml = junitparser.JUnitXml()

    # TestSuite is a class from junitparser that will contain the
    # results of running different TestCases between two shape_main
    # applications. A TestSuite contains a collection of TestCases.
    suite = junitparser.TestSuite(f"{name_publisher}---{name_subscriber}")

    timeout = 10
    now = datetime.now()

    t_suite_module = importlib.import_module(options['test_suite'])
    for test_suite_name, t_suite_dict in inspect.getmembers(t_suite_module):
        # getmembers returns all the members in the t_suite_module.
        # Then, 'type(t_suite) is dict' takes all the members that
        # are a dictionary. The only one that is not needed (it is not
        # a test_suite) is __builtins__, and it is skipped.
        if type(t_suite_dict) is dict and test_suite_name != '__builtins__':
            # check that the test_cases selected are in the test_suite and
            # exit the application if they are not.
            if not are_tests_in_test_suite(
                    t_suite_dict,
                    test_suite_name,
                    options['test_cases']):
                raise RuntimeError('Cannot process test cases.')
            if not are_tests_in_test_suite(
                    t_suite_dict,
                    test_suite_name,
                    options['test_cases_disabled']):
                raise RuntimeError('Disabled test cases not found.')

            for test_case_name, test_case_parameters in t_suite_dict.items():
                # TestCase is a class from junitparser whose attributes
                # are: name and result (OK, Failure, Error or Skipped).

                if options['test_cases_disabled'] is not None \
                        and test_case_name in options['test_cases_disabled']:
                    # if there are test cases disabled and the script is
                    # processing one of them, continue
                    print(f'Test Case {test_case_name} disabled.')
                    continue

                elif options['test_cases'] is not None \
                        and test_case_name not in options['test_cases']:
                    # if only specific test cases are enabled and the script
                    # is not processing one of them, continue
                    continue
                else:
                    # if the test case is processed
                    parameters = test_case_parameters['apps']
                    expected_codes = test_case_parameters['expected_codes']
                    if ('check_function' in test_case_parameters):
                        if callable(test_case_parameters['check_function']):
                            check_function = test_case_parameters['check_function']
                        else:
                            raise RuntimeError('Cannot process function of '
                                f'test case: {test_case_name}')
                    else:
                        check_function = no_check

                    assert(len(parameters) == len(expected_codes))

                    for element in parameters:
                        if not '-x ' in element:
                            element += f'-x {options["data_representation"]}'

                    case = junitparser.TestCase(f'{test_suite_name}_{test_case_name}')
                    now_test_case = datetime.now()
                    log_message(f'Running test: {test_case_name}', options['verbosity'])
                    run_test(name_executable_pub=options['publisher'],
                            name_executable_sub=options['subscriber'],
                            test_case=case,
                            parameters=parameters,
                            expected_codes=expected_codes,
                            verbosity=options['verbosity'],
                            timeout=timeout,
                            check_function=check_function)
                    case.time = (datetime.now() - now_test_case).total_seconds()
                    suite.add_testcase(case)

    suite.time = (datetime.now() - now).total_seconds()
    xml.add_testsuite(suite)

    xml.write(options['filename_report'])

if __name__ == '__main__':
    main()
